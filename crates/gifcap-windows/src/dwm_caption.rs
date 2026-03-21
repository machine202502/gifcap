//! Tint non-client area (title bar / border) on Windows 11+ via DWM.

use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR,
};

use crate::CaptureError;

fn hwnd_from_raw(raw: RawWindowHandle) -> Option<HWND> {
    match raw {
        RawWindowHandle::Win32(Win32WindowHandle { hwnd, .. }) => {
            Some(HWND(hwnd.get() as *mut std::ffi::c_void))
        }
        _ => None,
    }
}

/// `COLORREF` as `0x00bbggrr`.
const fn colorref(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

/// Sea-breeze title bar, matching the egui chrome. Fails silently on older Windows.
pub fn try_style_nonclient_sea_breeze(raw: RawWindowHandle) -> Result<(), CaptureError> {
    let hwnd = hwnd_from_raw(raw).ok_or(CaptureError::Gdi("not a Win32 window"))?;
    let caption = colorref(198, 230, 248);
    let border = colorref(120, 188, 218);
    let text = colorref(15, 65, 85);
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            &caption as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &border as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_TEXT_COLOR,
            &text as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
    Ok(())
}
