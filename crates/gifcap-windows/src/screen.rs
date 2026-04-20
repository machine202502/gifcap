use crate::{CaptureError, PhysicalRect};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

/// Returns the primary monitor bounds in physical pixels.
pub fn primary_monitor_rect() -> Result<PhysicalRect, CaptureError> {
    unsafe {
        let w = GetSystemMetrics(SM_CXSCREEN);
        let h = GetSystemMetrics(SM_CYSCREEN);
        if w <= 0 || h <= 0 {
            return Err(CaptureError::Gdi("GetSystemMetrics returned invalid bounds"));
        }
        Ok(PhysicalRect {
            x: 0,
            y: 0,
            width: w as u32,
            height: h as u32,
        })
    }
}
