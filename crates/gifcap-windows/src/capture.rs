use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetDC,
    ReleaseDC, SelectObject, BITMAPINFO, DIB_RGB_COLORS, HGDIOBJ, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

#[derive(Debug, Clone, Copy)]
pub struct PhysicalRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub enum CaptureError {
    Gdi(&'static str),
    Win32(windows::core::Error),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::Gdi(s) => write!(f, "{s}"),
            CaptureError::Win32(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CaptureError {}

impl From<windows::core::Error> for CaptureError {
    fn from(e: windows::core::Error) -> Self {
        CaptureError::Win32(e)
    }
}

/// Captures a screen rectangle into BGRA bytes (length `width * height * 4`), top row first.
pub fn capture_bgra(rect: PhysicalRect) -> Result<Vec<u8>, CaptureError> {
    let w = rect.width as i32;
    let h = rect.height as i32;
    if w <= 0 || h <= 0 {
        return Err(CaptureError::Gdi("invalid capture size"));
    }
    let len = (rect.width as usize)
        .checked_mul(rect.height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or(CaptureError::Gdi("size overflow"))?;
    let mut buf = vec![0u8; len];

    unsafe {
        let hwnd = GetDesktopWindow();
        let hdc_screen = GetDC(hwnd);
        if hdc_screen.is_invalid() {
            return Err(CaptureError::Gdi("GetDC failed"));
        }
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_invalid() {
            let _ = ReleaseDC(hwnd, hdc_screen);
            return Err(CaptureError::Gdi("CreateCompatibleDC failed"));
        }
        let hbm = CreateCompatibleBitmap(hdc_screen, w, h);
        if hbm.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(hwnd, hdc_screen);
            return Err(CaptureError::Gdi("CreateCompatibleBitmap failed"));
        }
        let old: HGDIOBJ = SelectObject(hdc_mem, hbm);

        if let Err(e) = BitBlt(hdc_mem, 0, 0, w, h, hdc_screen, rect.x, rect.y, SRCCOPY) {
            let _ = SelectObject(hdc_mem, old);
            let _ = DeleteObject(hbm);
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(hwnd, hdc_screen);
            return Err(e.into());
        }

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>()
            as u32;
        bmi.bmiHeader.biWidth = w;
        bmi.bmiHeader.biHeight = -h; // top-down DIB
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = 0; // BI_RGB

        let lines = GetDIBits(
            hdc_mem,
            hbm,
            0,
            rect.height,
            Some(buf.as_mut_ptr().cast()),
            &mut bmi,
            DIB_RGB_COLORS,
        );
        if lines == 0 {
            let _ = SelectObject(hdc_mem, old);
            let _ = DeleteObject(hbm);
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(hwnd, hdc_screen);
            return Err(CaptureError::Gdi("GetDIBits failed"));
        }

        let _ = SelectObject(hdc_mem, old);
        let _ = DeleteObject(hbm);
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(hwnd, hdc_screen);
    }

    Ok(buf)
}
