//! HWND helpers: window region = full frame + toolbar minus transparent viewport “hole”.

use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
use windows::Win32::Foundation::{BOOL, HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    ClientToScreen, CombineRgn, CreateRectRgn, DeleteObject, SetWindowRgn, RGN_DIFF, RGN_ERROR,
};
use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, GetWindowRect};

use crate::capture::{CaptureError, PhysicalRect};

fn hwnd_from_raw(raw: RawWindowHandle) -> Option<HWND> {
    match raw {
        RawWindowHandle::Win32(Win32WindowHandle { hwnd, .. }) => {
            Some(HWND(hwnd.get() as *mut std::ffi::c_void))
        }
        _ => None,
    }
}

/// Sets the window region so **title bar, borders, and toolbar** stay normal, while the capture
/// viewport becomes a true “hole” (desktop shows through, no HWND there).
///
/// Coordinates are Win32 **window** coordinates (origin = top-left of the window including frame).
///
/// `SetWindowRgn` takes ownership of the final region; temporary regions are deleted.
pub fn apply_toolbar_window_region(
    raw: RawWindowHandle,
    toolbar_bottom_client_px: i32,
    toolbar_at_top: bool,
) -> Result<(), CaptureError> {
    let hwnd = hwnd_from_raw(raw).ok_or(CaptureError::Gdi("not a Win32 window"))?;
    unsafe {
        let mut wr = RECT::default();
        GetWindowRect(hwnd, &mut wr)?;
        let mut cr = RECT::default();
        GetClientRect(hwnd, &mut cr)?;
        let cw = cr.right - cr.left;
        let ch = cr.bottom - cr.top;
        if cw <= 0 || ch <= 0 {
            return Err(CaptureError::Gdi("empty client area"));
        }

        let win_w = wr.right - wr.left;
        let win_h = wr.bottom - wr.top;

        let mut client0 = POINT { x: 0, y: 0 };
        if !ClientToScreen(hwnd, &mut client0).as_bool() {
            return Err(CaptureError::Gdi("ClientToScreen failed"));
        }
        let offset_x = client0.x - wr.left;
        let offset_y = client0.y - wr.top;

        let th = toolbar_bottom_client_px.clamp(0, ch);

        let full = CreateRectRgn(0, 0, win_w, win_h);
        if full.0.is_null() {
            return Err(CaptureError::Gdi("CreateRectRgn failed"));
        }

        // No separate viewport yet → whole window (decorations + client).
        if th <= 0 || th >= ch {
            if SetWindowRgn(hwnd, full, BOOL::from(true)) == 0 {
                let _ = DeleteObject(full);
                return Err(CaptureError::Gdi("SetWindowRgn failed"));
            }
            return Ok(());
        }

        let (hole_top, hole_bottom) = if toolbar_at_top {
            (offset_y + th, offset_y + ch)
        } else {
            (offset_y, offset_y + (ch - th))
        };
        let hole = CreateRectRgn(offset_x, hole_top, offset_x + cw, hole_bottom);
        if hole.0.is_null() {
            let _ = DeleteObject(full);
            return Err(CaptureError::Gdi("CreateRectRgn (hole) failed"));
        }

        let combined = CreateRectRgn(0, 0, 0, 0);
        if combined.0.is_null() {
            let _ = DeleteObject(full);
            let _ = DeleteObject(hole);
            return Err(CaptureError::Gdi("CreateRectRgn (combined) failed"));
        }

        let crgn = CombineRgn(combined, full, hole, RGN_DIFF);
        let _ = DeleteObject(full);
        let _ = DeleteObject(hole);

        if crgn == RGN_ERROR {
            let _ = DeleteObject(combined);
            return Err(CaptureError::Gdi("CombineRgn failed"));
        }

        if SetWindowRgn(hwnd, combined, BOOL::from(true)) == 0 {
            let _ = DeleteObject(combined);
            return Err(CaptureError::Gdi("SetWindowRgn failed"));
        }
        Ok(())
    }
}

/// Screen rectangle of the capture viewport relative to toolbar position.
pub fn physical_viewport_rect(
    raw: RawWindowHandle,
    toolbar_bottom_client_px: i32,
    toolbar_at_top: bool,
) -> Result<PhysicalRect, CaptureError> {
    let hwnd = hwnd_from_raw(raw).ok_or(CaptureError::Gdi("not a Win32 window"))?;
    unsafe {
        let mut cr = RECT::default();
        GetClientRect(hwnd, &mut cr)?;
        let cw = cr.right - cr.left;
        let ch = cr.bottom - cr.top;
        let th = toolbar_bottom_client_px.clamp(0, ch);
        let cap_h = ch - th;
        if cap_h < 1 || cw < 1 {
            return Err(CaptureError::Gdi("viewport too small"));
        }
        let y0 = if toolbar_at_top { th } else { 0 };
        let mut pt = POINT { x: 0, y: y0 };
        if !ClientToScreen(hwnd, &mut pt).as_bool() {
            return Err(CaptureError::Gdi("ClientToScreen failed"));
        }
        Ok(PhysicalRect {
            x: pt.x,
            y: pt.y,
            width: cw as u32,
            height: cap_h as u32,
        })
    }
}
