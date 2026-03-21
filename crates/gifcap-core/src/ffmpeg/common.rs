use std::sync::Once;

use ffmpeg_next as ffmpeg;
use ffmpeg::{frame, util::format::Pixel};

use crate::CoreError;

static FFMPEG_INIT: Once = Once::new();

pub(super) fn ensure_ffmpeg_init() {
    FFMPEG_INIT.call_once(|| {
        if let Err(e) = ffmpeg::init() {
            eprintln!("ffmpeg::init failed: {e:?}");
        }
    });
}

pub(super) fn map_ffmpeg(e: ffmpeg::Error) -> CoreError {
    CoreError::Export(format!("ffmpeg: {e:?}"))
}

/// Copy tightly packed top-down BGRA (`width * 4` per row) into a `Video` frame.
pub(super) fn fill_bgra_frame(
    bgra: &[u8],
    width: u32,
    height: u32,
    dst: &mut frame::Video,
) -> Result<(), CoreError> {
    let line = width as usize * 4;
    let expected = line * height as usize;
    if bgra.len() != expected {
        return Err(CoreError::Gif(format!(
            "frame size mismatch: got {} expected {}",
            bgra.len(),
            expected
        )));
    }
    unsafe {
        if dst.is_empty()
            || dst.width() != width
            || dst.height() != height
            || dst.format() != Pixel::BGRA
        {
            dst.alloc(Pixel::BGRA, width, height);
        }
    }
    let stride = dst.stride(0) as usize;
    let plane = dst.data_mut(0);
    for y in 0..height as usize {
        let s = y * line;
        let d = y * stride;
        plane[d..d + line].copy_from_slice(&bgra[s..s + line]);
    }
    Ok(())
}
