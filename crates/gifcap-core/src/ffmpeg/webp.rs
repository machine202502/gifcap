use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use ffmpeg_next as ffmpeg;
use ffmpeg::{
    codec, encoder, format,
    frame,
    media::Type,
    software::scaling,
    util::format::Pixel,
    util::mathematics::Rescale,
    Codec, Dictionary, Packet, Rational,
};

use crate::CoreError;

use super::common::{ensure_ffmpeg_init, map_ffmpeg};

#[inline]
fn check_cancel(cancel: Option<&AtomicBool>) -> Result<(), CoreError> {
    if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
        Err(CoreError::Cancelled)
    } else {
        Ok(())
    }
}

fn enc_pts_for_frame(
    src: &frame::Video,
    ist_tb: Rational,
    enc_tb: Rational,
    ms_tb: Rational,
    next_pts_ms: &mut i64,
    delay_ms: i64,
) -> i64 {
    match src.pts() {
        Some(p) => p.rescale(ist_tb, enc_tb),
        None => {
            let v = *next_pts_ms;
            *next_pts_ms += delay_ms;
            v.rescale(ms_tb, enc_tb)
        }
    }
}

fn encode_webp_frame(
    decoded: &mut frame::Video,
    encoder: &mut encoder::video::Encoder,
    output: &mut format::context::Output,
    pkt: &mut Packet,
    ist_tb: Rational,
    enc_tb: Rational,
    st_tb: Rational,
    ms_tb: Rational,
    next_pts_ms: &mut i64,
    delay_ms: i64,
) -> Result<(), CoreError> {
    let ep = enc_pts_for_frame(
        decoded,
        ist_tb,
        enc_tb,
        ms_tb,
        next_pts_ms,
        delay_ms,
    );
    decoded.set_pts(Some(ep));
    encoder.send_frame(decoded).map_err(map_ffmpeg)?;
    flush_enc_packets(encoder, output, pkt, enc_tb, st_tb)?;
    Ok(())
}

fn flush_enc_packets(
    enc: &mut encoder::video::Encoder,
    output: &mut format::context::Output,
    pkt: &mut Packet,
    enc_tb: Rational,
    st_tb: Rational,
) -> Result<(), CoreError> {
    while enc.receive_packet(pkt).is_ok() {
        if pkt.size() == 0 {
            continue;
        }
        pkt.set_stream(0);
        pkt.rescale_ts(enc_tb, st_tb);
        pkt.write_interleaved(output).map_err(map_ffmpeg)?;
    }
    Ok(())
}

/// **MP4 → animated WebP** using linked FFmpeg (same as capture).
///
/// **Looping:** the WebP muxer’s default `loop` is `1`, and FFmpeg patches the file on close —
/// that overrides libwebp’s default infinite loop. We pass **`loop=0`** to the muxer so the
/// anim repeats forever (WebP: 0 = infinite).
///
/// **Speed:** when the decoder already outputs `YUV420P` (typical for H.264), we encode WebP
/// without BGRA conversion. `compression_level` is fixed; **`quality`** comes from the caller.
///
/// **`cancel`:** polled between frames; on `true` returns [`CoreError::Cancelled`] and removes a partial output file.
pub(crate) fn convert_mp4_to_webp(
    input_mp4: &Path,
    output_webp: &Path,
    quality: u8,
    cancel: Option<&AtomicBool>,
) -> Result<(), CoreError> {
    let quality = quality.clamp(1, 100);
    if !input_mp4.is_file() {
        return Err(CoreError::NoFrames);
    }
    if let Some(parent) = output_webp.parent() {
        crate::paths::ensure_dir(parent)?;
    }

    ensure_ffmpeg_init();
    check_cancel(cancel)?;

    let inner = (|| -> Result<(), CoreError> {
    let mut input = format::input(input_mp4).map_err(map_ffmpeg)?;
    let in_stream = input
        .streams()
        .best(Type::Video)
        .ok_or_else(|| CoreError::Export("MP4 has no video stream".into()))?;
    let in_stream_idx = in_stream.index();
    let ist_tb = in_stream.time_base();

    let mut decoder = codec::context::Context::from_parameters(in_stream.parameters())
        .map_err(map_ffmpeg)?
        .decoder()
        .video()
        .map_err(map_ffmpeg)?;

    let width = decoder.width();
    let height = decoder.height();
    let enc_w = width & !1;
    let enc_h = height & !1;
    if enc_w < 2 || enc_h < 2 {
        return Err(CoreError::Export("video dimensions too small for WebP".into()));
    }

    let dec_fmt = decoder.format();
    let use_yuv =
        dec_fmt == Pixel::YUV420P && enc_w == width && enc_h == height;

    let mut fr = in_stream.avg_frame_rate();
    if fr.numerator() <= 0 || fr.denominator() <= 0 {
        fr = Rational(30, 1);
    }
    let delay_ms = ((1000.0 * fr.denominator() as f64 / fr.numerator() as f64).round() as i64).max(1);

    let codec: Codec = encoder::find(codec::Id::WEBP)
        .or_else(|| encoder::find_by_name("libwebp_anim"))
        .or_else(|| encoder::find_by_name("libwebp"))
        .ok_or_else(|| {
            CoreError::Export(
                "FFmpeg has no WebP encoder (libwebp). Rebuild FFmpeg with WebP support.".into(),
            )
        })?;

    let ms_tb = Rational(1, 1000);
    let mut output = format::output(output_webp).map_err(map_ffmpeg)?;
    let mut out_stream = output.add_stream(codec).map_err(map_ffmpeg)?;
    out_stream.set_time_base(ms_tb);
    out_stream.set_avg_frame_rate(fr);

    let enc_pixel = if use_yuv {
        Pixel::YUV420P
    } else {
        Pixel::BGRA
    };

    let mut enc_ctx = codec::context::Context::new_with_codec(codec)
        .encoder()
        .video()
        .map_err(map_ffmpeg)?;
    enc_ctx.set_width(enc_w);
    enc_ctx.set_height(enc_h);
    enc_ctx.set_format(enc_pixel);
    enc_ctx.set_time_base(ms_tb);
    enc_ctx.set_frame_rate(Some(fr));

    let mut opts = Dictionary::new();
    opts.set("lossless", "0");
    opts.set("quality", &quality.to_string());
    opts.set("compression_level", "2");
    // No preset: use quality + method only (avoids extra preset work).

    let mut encoder = enc_ctx.open_with(opts).map_err(map_ffmpeg)?;
    let opened_pixel = encoder.format();
    if opened_pixel != enc_pixel {
        return Err(CoreError::Export(format!(
            "WebP encoder uses {opened_pixel:?}, expected {enc_pixel:?}"
        )));
    }

    out_stream.set_parameters(&encoder);
    drop(out_stream);

    // Critical: muxer default `loop` is 1 → trailer overwrites ANIM to play once.
    // `loop=0` = infinite repeat per WebP / FFmpeg muxer docs.
    let mut mux_opts = Dictionary::new();
    mux_opts.set("loop", "0");
    let _ = output.write_header_with(mux_opts).map_err(map_ffmpeg)?;

    let st_tb = output
        .stream(0)
        .ok_or_else(|| CoreError::Export("muxer: missing stream 0".into()))?
        .time_base();
    let enc_tb = encoder.time_base();

    let mut scaler = if use_yuv {
        None
    } else {
        Some(
            scaling::Context::get(
                dec_fmt,
                width,
                height,
                opened_pixel,
                enc_w,
                enc_h,
                scaling::flag::Flags::BILINEAR | scaling::flag::Flags::FULL_CHR_H_INT,
            )
            .map_err(map_ffmpeg)?,
        )
    };

    let mut decoded = frame::Video::empty();
    let mut bgra = frame::Video::new(Pixel::BGRA, enc_w, enc_h);
    let mut pkt = Packet::empty();
    let mut next_pts_ms: i64 = 0;

    for (stream, packet) in input.packets() {
        if stream.index() != in_stream_idx {
            continue;
        }
        check_cancel(cancel)?;
        decoder.send_packet(&packet).map_err(map_ffmpeg)?;
        while decoder.receive_frame(&mut decoded).is_ok() {
            check_cancel(cancel)?;
            if use_yuv {
                encode_webp_frame(
                    &mut decoded,
                    &mut encoder,
                    &mut output,
                    &mut pkt,
                    ist_tb,
                    enc_tb,
                    st_tb,
                    ms_tb,
                    &mut next_pts_ms,
                    delay_ms,
                )?;
            } else {
                let sc = scaler.as_mut().expect("scaler when not use_yuv");
                sc.run(&decoded, &mut bgra).map_err(map_ffmpeg)?;
                let ep = enc_pts_for_frame(
                    &decoded,
                    ist_tb,
                    enc_tb,
                    ms_tb,
                    &mut next_pts_ms,
                    delay_ms,
                );
                bgra.set_pts(Some(ep));
                encoder.send_frame(&bgra).map_err(map_ffmpeg)?;
                flush_enc_packets(&mut encoder, &mut output, &mut pkt, enc_tb, st_tb)?;
            }
        }
    }

    decoder.send_eof().map_err(map_ffmpeg)?;
    while decoder.receive_frame(&mut decoded).is_ok() {
        check_cancel(cancel)?;
        if use_yuv {
            encode_webp_frame(
                &mut decoded,
                &mut encoder,
                &mut output,
                &mut pkt,
                ist_tb,
                enc_tb,
                st_tb,
                ms_tb,
                &mut next_pts_ms,
                delay_ms,
            )?;
        } else {
            let sc = scaler.as_mut().expect("scaler when not use_yuv");
            sc.run(&decoded, &mut bgra).map_err(map_ffmpeg)?;
            let ep = enc_pts_for_frame(
                &decoded,
                ist_tb,
                enc_tb,
                ms_tb,
                &mut next_pts_ms,
                delay_ms,
            );
            bgra.set_pts(Some(ep));
            encoder.send_frame(&bgra).map_err(map_ffmpeg)?;
            flush_enc_packets(&mut encoder, &mut output, &mut pkt, enc_tb, st_tb)?;
        }
    }

    check_cancel(cancel)?;
    encoder.send_eof().map_err(map_ffmpeg)?;
    flush_enc_packets(&mut encoder, &mut output, &mut pkt, enc_tb, st_tb)?;

    check_cancel(cancel)?;
    output.write_trailer().map_err(map_ffmpeg)?;

    if !output_webp.is_file() {
        return Err(CoreError::Export(
            "WebP export finished but output file is missing.".into(),
        ));
    }

    Ok(())
    })();

    if matches!(&inner, Err(CoreError::Cancelled)) {
        let _ = std::fs::remove_file(output_webp);
    }
    inner
}
