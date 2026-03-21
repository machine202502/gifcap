use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use ffmpeg_next as ffmpeg;
use ffmpeg::{
    codec,
    codec::flag::Flags as CodecFlags,
    encoder,
    format,
    format::flag::Flags as FmtFlags,
    frame,
    software::scaling,
    util::format::Pixel,
    util::mathematics::Rescale,
    Codec, Dictionary, Packet, Rational,
};

use crate::CoreError;

use super::common::{fill_bgra_frame, map_ffmpeg};

/// Prefer software / quality encoders first. `avcodec_find_encoder(H264)` on Windows often picks
/// `h264_mf` (Media Foundation) with weak defaults → blurry screen captures.
fn find_mp4_video_codec() -> Option<Codec> {
    encoder::find_by_name("libx264")
        .or_else(|| encoder::find_by_name("libopenh264"))
        .or_else(|| encoder::find(codec::Id::H264))
        .or_else(|| encoder::find_by_name("h264_nvenc"))
        .or_else(|| encoder::find_by_name("h264_amf"))
        .or_else(|| encoder::find_by_name("h264_qsv"))
        .or_else(|| encoder::find_by_name("h264_mf"))
        .or_else(|| encoder::find(codec::Id::MPEG4))
}

pub(super) fn writer_loop(
    rx: Receiver<Vec<u8>>,
    return_tx: Sender<Vec<u8>>,
    output_path: PathBuf,
    width: u32,
    height: u32,
    fps: f64,
    quality: u8,
) -> Result<(), CoreError> {
    let q = quality.clamp(1, 100) as i32;
    let codec = find_mp4_video_codec().ok_or_else(|| {
        CoreError::Export(
            "FFmpeg has no H.264/MPEG-4 encoder for MP4 (unusual build). \
             vcpkg `ffmpeg` default usually includes at least MPEG-4 Part 2."
                .into(),
        )
    })?;

    let cname = codec.name().to_owned();
    let is_mpeg4 = codec.id() == codec::Id::MPEG4;

    let fps_i = fps.round().clamp(1.0, 120.0) as i32;
    let fps_i = fps_i.max(1);
    let fps_tb = Rational(1, fps_i);

    let mut output = format::output(&output_path).map_err(map_ffmpeg)?;
    let needs_global_header = output
        .format()
        .flags()
        .contains(FmtFlags::GLOBAL_HEADER);
    let mut stream = output.add_stream(codec).map_err(map_ffmpeg)?;
    stream.set_time_base(fps_tb);
    stream.set_avg_frame_rate(Rational(fps_i, 1));

    let mut enc_ctx = codec::context::Context::new_with_codec(codec)
        .encoder()
        .video()
        .map_err(map_ffmpeg)?;

    enc_ctx.set_width(width);
    enc_ctx.set_height(height);
    enc_ctx.set_format(Pixel::YUV420P);
    enc_ctx.set_time_base(fps_tb);
    enc_ctx.set_frame_rate(Some(Rational(fps_i, 1)));
    enc_ctx.set_max_b_frames(0);
    enc_ctx.set_gop((fps_i as u32).saturating_mul(2).max(1));

    if needs_global_header {
        enc_ctx.set_flags(CodecFlags::GLOBAL_HEADER);
    }

    if is_mpeg4 {
        let br_base = (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(fps_i as usize)
            / 2;
        let scale = (50 + q as usize).min(200);
        let br = br_base
            .saturating_mul(scale)
            .saturating_div(100)
            .max(500_000)
            .min(100_000_000);
        enc_ctx.set_bit_rate(br);
    }

    let mut opts = Dictionary::new();
    match cname.as_str() {
        "libx264" => {
            opts.set("preset", "fast");
            // q=100 → CRF ~17 (previous default); lower q → higher CRF (smaller files).
            let crf = 51 - (q * 34 / 100);
            let crf = crf.clamp(15, 40);
            opts.set("crf", &crf.to_string());
        }
        "libopenh264" => {
            opts.set("profile", "baseline");
        }
        "h264_mf" => {
            opts.set("rate_control", "quality");
            // q=100 → quality 90 (previous default).
            let mq = ((q as u32 * 90 + 50) / 100).clamp(1, 100);
            opts.set("quality", &mq.to_string());
        }
        _ => {}
    }

    let mut encoder = enc_ctx.open_with(opts).map_err(map_ffmpeg)?;
    let enc_pixel = encoder.format();
    stream.set_parameters(&encoder);
    drop(stream);
    output.write_header().map_err(map_ffmpeg)?;
    let st_tb = output
        .stream(0)
        .ok_or_else(|| CoreError::Export("muxer: missing stream 0".into()))?
        .time_base();
    let enc_tb = encoder.time_base();

    let mut scaler = scaling::Context::get(
        Pixel::BGRA,
        width,
        height,
        enc_pixel,
        width,
        height,
        scaling::flag::Flags::BICUBIC | scaling::flag::Flags::FULL_CHR_H_INT,
    )
    .map_err(map_ffmpeg)?;

    let mut bgra_src = frame::Video::new(Pixel::BGRA, width, height);
    let mut enc_frame = frame::Video::empty();

    let mut frame_idx: i64 = 0;
    let mut pkt = Packet::empty();

    let flush_packets = |venc: &mut encoder::video::Encoder,
                         out: &mut format::context::Output,
                         pkt: &mut Packet|
     -> Result<(), CoreError> {
        while venc.receive_packet(pkt).is_ok() {
            if pkt.size() == 0 {
                continue;
            }
            pkt.set_stream(0);
            pkt.rescale_ts(enc_tb, st_tb);
            pkt.write_interleaved(out).map_err(map_ffmpeg)?;
        }
        Ok(())
    };

    loop {
        let buf = match rx.recv() {
            Ok(b) => b,
            Err(_) => break,
        };

        fill_bgra_frame(&buf, width, height, &mut bgra_src)?;
        scaler
            .run(&bgra_src, &mut enc_frame)
            .map_err(map_ffmpeg)?;

        let pts = frame_idx.rescale(fps_tb, enc_tb);
        enc_frame.set_pts(Some(pts));
        frame_idx = frame_idx.saturating_add(1);

        encoder.send_frame(&enc_frame).map_err(map_ffmpeg)?;
        flush_packets(&mut encoder, &mut output, &mut pkt)?;

        let _ = return_tx.send(buf);
    }

    encoder.send_eof().map_err(map_ffmpeg)?;
    flush_packets(&mut encoder, &mut output, &mut pkt)?;

    output.write_trailer().map_err(map_ffmpeg)?;
    Ok(())
}
