use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use ffmpeg_next as ffmpeg;
use ffmpeg::{
    codec, encoder, filter, format, frame, util::format::Pixel, util::mathematics::Rescale,
    Dictionary, Packet, Rational,
};

use crate::CoreError;

use super::common::{fill_bgra_frame, map_ffmpeg};

/// GIF delay is stored in centiseconds. Choose the delay whose playback rate is closest to `fps`.
fn best_delay_centisecs(fps: f64) -> i32 {
    if !fps.is_finite() || fps <= 0.0 {
        return 10;
    }
    let fps = fps.clamp(0.1, 120.0);
    let mut best_d = 10i32;
    let mut best_err = f64::MAX;
    for d in 2..=500i32 {
        let err = (100.0 / f64::from(d) - fps).abs();
        if err < best_err {
            best_err = err;
            best_d = d;
        }
    }
    best_d
}

fn even(value: u32) -> u32 {
    (value & !1).max(2)
}

/// Same palette path ezcap uses for GIF, applied to one frame so a long recording stays on disk.
/// `diff_mode=rectangle` keeps unchanged screen regions out of the file; sierra2 keeps edges clean.
fn palettize(
    src: &frame::Video,
    out_w: u32,
    out_h: u32,
    colors: u32,
) -> Result<frame::Video, CoreError> {
    let mut graph = filter::Graph::new();
    let args = format!(
        "video_size={}x{}:pix_fmt=bgra:time_base=1/100:pixel_aspect=1/1",
        src.width(),
        src.height()
    );
    let buffer = filter::find("buffer").ok_or_else(|| {
        CoreError::Export("FFmpeg buffer filter is unavailable".into())
    })?;
    let sink = filter::find("buffersink").ok_or_else(|| {
        CoreError::Export("FFmpeg buffersink filter is unavailable".into())
    })?;
    graph.add(&buffer, "in", &args).map_err(map_ffmpeg)?;
    graph.add(&sink, "out", "").map_err(map_ffmpeg)?;
    let spec = format!(
        "scale={out_w}:{out_h}:flags=lanczos,split[a][b];[a]palettegen=max_colors={colors}:stats_mode=single[p];[b][p]paletteuse=dither=sierra2:diff_mode=rectangle"
    );
    graph
        .output("in", 0)
        .map_err(map_ffmpeg)?
        .input("out", 0)
        .map_err(map_ffmpeg)?
        .parse(&spec)
        .map_err(map_ffmpeg)?;
    graph.validate().map_err(map_ffmpeg)?;
    {
        let mut input = graph
            .get("in")
            .ok_or_else(|| CoreError::Export("GIF filter has no input".into()))?;
        input.source().add(src).map_err(map_ffmpeg)?;
        input.source().flush().map_err(map_ffmpeg)?;
    }
    let mut out = frame::Video::empty();
    graph
        .get("out")
        .ok_or_else(|| CoreError::Export("GIF filter has no output".into()))?
        .sink()
        .frame(&mut out)
        .map_err(map_ffmpeg)?;
    Ok(out)
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
    let colors = 64 + u32::from(q) * 192 / 100;
    let (out_w, out_h) = (even(width), even(height));
    let delay_cs_i32 = best_delay_centisecs(fps);
    let delay_cs = i64::from(delay_cs_i32);
    let cs_tb = Rational(1, 100);
    let fps_r = Rational(100, delay_cs_i32);

    let codec = encoder::find(codec::Id::GIF).ok_or_else(|| {
        CoreError::Export("FFmpeg has no GIF encoder. Rebuild FFmpeg with avcodec.".into())
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
    enc_ctx.set_format(Pixel::PAL8);
    enc_ctx.set_time_base(cs_tb);
    enc_ctx.set_frame_rate(Some(fps_r));

    let mut encoder = enc_ctx.open_with(Dictionary::new()).map_err(map_ffmpeg)?;
    stream.set_parameters(&encoder);
    drop(stream);
    output.write_header().map_err(map_ffmpeg)?;
    let st_tb = output
        .stream(0)
        .ok_or_else(|| CoreError::Export("muxer: missing stream 0".into()))?
        .time_base();
    let enc_tb = encoder.time_base();

    let mut bgra_src = frame::Video::new(Pixel::BGRA, width, height);
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
        let mut enc_frame = palettize(&bgra_src, out_w, out_h, colors)?;
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
