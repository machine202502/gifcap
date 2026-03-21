use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use ffmpeg_next as ffmpeg;
use ffmpeg::{
    codec, encoder, format, frame, software::scaling, util::format::Pixel,
    util::mathematics::Rescale, Dictionary, Packet, Rational,
};

use crate::CoreError;

use super::common::{fill_bgra_frame, map_ffmpeg};

/// GIF frame delay is stored in **centiseconds** (1/100 s). Pick `d ∈ [2, 500]` minimizing
/// |100/d − fps| so playback CFR matches the UI FPS better than `round(100/fps)`.
fn best_delay_centisecs(fps: f64) -> i32 {
    if !fps.is_finite() || fps <= 0.0 {
        return 10;
    }
    let fps = fps.clamp(0.1, 120.0);
    let mut best_d = 10i32;
    let mut best_err = f64::MAX;
    for d in 2..=500i32 {
        let eff = 100.0 / f64::from(d);
        let err = (eff - fps).abs();
        if err < best_err {
            best_err = err;
            best_d = d;
        }
    }
    best_d
}

/// Output size for GIF: fewer pixels at lower quality (main lever for file size). Uses √(
/// *q*/100) per axis so pixel count roughly tracks *q* (e.g. *q*=25 → half side, ~¼ pixels).
fn gif_output_dimensions(width: u32, height: u32, q: u8) -> (u32, u32) {
    let q = q.clamp(1, 100) as f64;
    let s = (q / 100.0).sqrt().clamp(0.2, 1.0);
    let mut out_w = ((f64::from(width) * s).round() as u32).clamp(2, width);
    let mut out_h = ((f64::from(height) * s).round() as u32).clamp(2, height);
    out_w -= out_w % 2;
    out_h -= out_h % 2;
    (out_w.max(2), out_h.max(2))
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
    let q = quality.clamp(1, 100);
    let (out_w, out_h) = gif_output_dimensions(width, height, q);
    let scale_flags = if q >= 50 {
        scaling::flag::Flags::BICUBIC | scaling::flag::Flags::FULL_CHR_H_INT
    } else {
        scaling::flag::Flags::BILINEAR | scaling::flag::Flags::FULL_CHR_H_INT
    };
    let delay_cs_i32 = best_delay_centisecs(fps);
    let delay_cs = delay_cs_i32 as i64;
    let cs_tb = Rational(1, 100);
    let fps_r = Rational(100, delay_cs_i32);

    let codec = encoder::find(codec::Id::GIF).ok_or_else(|| {
        CoreError::Export(
            "FFmpeg has no GIF encoder (unusual). Rebuild FFmpeg with avcodec.".into(),
        )
    })?;

    let mut output = format::output(&output_path).map_err(map_ffmpeg)?;
    let mut stream = output.add_stream(codec).map_err(map_ffmpeg)?;
    stream.set_time_base(cs_tb);
    stream.set_avg_frame_rate(fps_r);

    let mut enc_ctx = codec::context::Context::new_with_codec(codec)
        .encoder()
        .video()
        .map_err(map_ffmpeg)?;
    enc_ctx.set_width(out_w);
    enc_ctx.set_height(out_h);
    enc_ctx.set_format(Pixel::RGB8);
    enc_ctx.set_time_base(cs_tb);
    enc_ctx.set_frame_rate(Some(fps_r));

    let mut opts = Dictionary::new();
    opts.set("gifflags", "0");

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
        out_w,
        out_h,
        scale_flags,
    )
    .map_err(map_ffmpeg)?;

    let mut bgra_src = frame::Video::new(Pixel::BGRA, width, height);
    let mut enc_frame = frame::Video::empty();

    let mut t_cs: i64 = 0;
    let mut pkt = Packet::empty();
    let d_st = delay_cs.rescale(cs_tb, st_tb);
    let mut mux_pts: i64 = 0;

    let flush_packets = |venc: &mut encoder::video::Encoder,
                         out: &mut format::context::Output,
                         pkt: &mut Packet,
                         mux_pts: &mut i64|
     -> Result<(), CoreError> {
        while venc.receive_packet(pkt).is_ok() {
            if pkt.size() == 0 {
                continue;
            }
            pkt.set_stream(0);
            pkt.rescale_ts(enc_tb, st_tb);
            pkt.set_pts(Some(*mux_pts));
            pkt.set_duration(d_st);
            *mux_pts = mux_pts.saturating_add(d_st);
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

        let pts = t_cs.rescale(cs_tb, enc_tb);
        enc_frame.set_pts(Some(pts));
        t_cs = t_cs.saturating_add(delay_cs);

        encoder.send_frame(&enc_frame).map_err(map_ffmpeg)?;
        flush_packets(&mut encoder, &mut output, &mut pkt, &mut mux_pts)?;

        let _ = return_tx.send(buf);
    }

    encoder.send_eof().map_err(map_ffmpeg)?;
    flush_packets(&mut encoder, &mut output, &mut pkt, &mut mux_pts)?;

    output.write_trailer().map_err(map_ffmpeg)?;
    Ok(())
}
