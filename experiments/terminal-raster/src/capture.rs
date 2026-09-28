use image::RgbaImage;
use std::path::Path;
use windows::{
  Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    Storage::Xps::*,
    UI::{HiDpi::*, WindowsAndMessaging::*},
  },
  core::*,
};

struct Search {
  title: String,
  matches: Vec<HWND>,
}
unsafe extern "system" fn visit(hwnd: HWND, data: LPARAM) -> BOOL {
  // EnumWindows passes our live Search pointer synchronously.
  unsafe {
    let search = &mut *(data.0 as *mut Search);
    let mut title = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut title);
    if IsWindowVisible(hwnd).as_bool()
      && String::from_utf16_lossy(&title[..len as usize]).contains(&search.title)
    {
      search.matches.push(hwnd);
    }
  }
  BOOL(1)
}

struct Surface {
  hwnd: HWND,
  screen: HDC,
  memory: HDC,
  bitmap: HBITMAP,
  previous: HGDIOBJ,
}
impl Drop for Surface {
  fn drop(&mut self) {
    unsafe {
      if !self.previous.is_invalid() {
        let _ = SelectObject(self.memory, self.previous);
      }
      if !self.bitmap.is_invalid() {
        let _ = DeleteObject(self.bitmap.into());
      }
      if !self.memory.is_invalid() {
        let _ = DeleteDC(self.memory);
      }
      if !self.screen.is_invalid() {
        let _ = ReleaseDC(Some(self.hwnd), self.screen);
      }
    }
  }
}

pub fn capture(title: &str, path: &Path) -> std::result::Result<(), Box<dyn std::error::Error>> {
  if title.len() < 8 {
    return Err("use a unique title of at least 8 bytes; broad window captures are refused".into());
  }
  unsafe {
    SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
    let mut search = Search {
      title: title.into(),
      matches: Vec::new(),
    };
    EnumWindows(Some(visit), LPARAM(&mut search as *mut Search as isize))?;
    if search.matches.len() != 1 {
      return Err(
        format!(
          "expected one matching visible window, found {}",
          search.matches.len()
        )
        .into(),
      );
    }
    let hwnd = search.matches[0];
    let mut rect = RECT::default();
    GetWindowRect(hwnd, &mut rect)?;
    let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
    if w <= 0 || h <= 0 || w > 8192 || h > 8192 {
      return Err("invalid window size".into());
    }
    let mut surface = Surface {
      hwnd,
      screen: GetDC(Some(hwnd)),
      memory: HDC::default(),
      bitmap: HBITMAP::default(),
      previous: HGDIOBJ::default(),
    };
    if surface.screen.is_invalid() {
      return Err("GetDC failed".into());
    }
    surface.memory = CreateCompatibleDC(Some(surface.screen));
    surface.bitmap = CreateCompatibleBitmap(surface.screen, w, h);
    if surface.memory.is_invalid() || surface.bitmap.is_invalid() {
      return Err("capture allocation failed".into());
    }
    surface.previous = SelectObject(surface.memory, surface.bitmap.into());
    if surface.previous.is_invalid() {
      return Err("SelectObject failed".into());
    }
    if !PrintWindow(hwnd, surface.memory, PRINT_WINDOW_FLAGS(2)).as_bool() {
      return Err("PrintWindow failed".into());
    }
    // GetDIBits requires that the bitmap is not selected into a DC.
    let _ = SelectObject(surface.memory, surface.previous);
    surface.previous = HGDIOBJ::default();
    let mut info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: w,
        biHeight: -h,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
      },
      ..Default::default()
    };
    let mut bytes = vec![0u8; w as usize * h as usize * 4];
    if GetDIBits(
      surface.memory,
      surface.bitmap,
      0,
      h as u32,
      Some(bytes.as_mut_ptr().cast()),
      &mut info,
      DIB_RGB_COLORS,
    ) != h
    {
      return Err("GetDIBits incomplete".into());
    }
    for pixel in bytes.chunks_exact_mut(4) {
      pixel.swap(0, 2);
      pixel[3] = 255;
    }
    RgbaImage::from_raw(w as u32, h as u32, bytes)
      .ok_or("invalid capture buffer")?
      .save(path)?;
    std::fs::write(
      path.with_extension("json"),
      serde_json::to_vec_pretty(&serde_json::json!({
          "title_match":title,"dpi":GetDpiForWindow(hwnd),"width":w,"height":h,
          "method":"PrintWindow PW_RENDERFULLCONTENT; only unique target window; no desktop capture"
      }))?,
    )?;
  }
  Ok(())
}

pub fn close(title: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
  if !title.starts_with("TG-B7-LAB-") {
    return Err("only TG-B7-LAB- test windows may be closed".into());
  }
  unsafe {
    let mut search = Search {
      title: title.into(),
      matches: Vec::new(),
    };
    EnumWindows(Some(visit), LPARAM(&mut search as *mut Search as isize))?;
    if search.matches.len() != 1 {
      return Err("expected one unique lab window".into());
    }
    PostMessageW(Some(search.matches[0]), WM_CLOSE, WPARAM(0), LPARAM(0))?;
  }
  Ok(())
}
