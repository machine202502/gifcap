//! Windows-specific capture helpers. Other platforms can add sibling crates later.
//!
//! Uses GDI `BitBlt` into a DIB and returns tightly packed BGRA (bottom-up or top-down per DIB setup).

#[cfg(windows)]
mod capture;
#[cfg(windows)]
mod dpi;
#[cfg(windows)]
mod dwm_caption;
#[cfg(windows)]
mod win32_window;

#[cfg(windows)]
pub use capture::{capture_bgra, CaptureError, PhysicalRect};
#[cfg(windows)]
pub use dpi::init_dpi_awareness;
#[cfg(windows)]
pub use dwm_caption::try_style_nonclient_sea_breeze;
#[cfg(windows)]
pub use win32_window::{apply_toolbar_window_region, physical_viewport_rect};

#[cfg(not(windows))]
#[derive(Debug, Clone, Copy)]
pub struct PhysicalRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[cfg(not(windows))]
#[derive(Debug)]
pub struct CaptureError(&'static str);

#[cfg(not(windows))]
impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

#[cfg(not(windows))]
impl std::error::Error for CaptureError {}

#[cfg(not(windows))]
pub fn init_dpi_awareness() {}

#[cfg(not(windows))]
pub fn capture_bgra(_rect: PhysicalRect) -> Result<Vec<u8>, CaptureError> {
    Err(CaptureError("gifcap-windows only builds capture on Windows"))
}

#[cfg(not(windows))]
use raw_window_handle::RawWindowHandle;

#[cfg(not(windows))]
pub fn apply_toolbar_window_region(
    _raw: RawWindowHandle,
    _toolbar_bottom_client_px: i32,
) -> Result<(), CaptureError> {
    Err(CaptureError("Windows only"))
}

#[cfg(not(windows))]
pub fn try_style_nonclient_sea_breeze(_raw: RawWindowHandle) -> Result<(), CaptureError> {
    Err(CaptureError("Windows only"))
}

#[cfg(not(windows))]
pub fn physical_viewport_rect(
    _raw: RawWindowHandle,
    _toolbar_bottom_client_px: i32,
) -> Result<PhysicalRect, CaptureError> {
    Err(CaptureError("Windows only"))
}
