use std::mem::MaybeUninit;
use std::io::Read;
use std::fs::File;
use std::ffi::OsStr;
use std::error::Error;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use base64::engine::general_purpose;
use base64::Engine;
use image::RgbaImage;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::DeleteObject;
use windows::Win32::Graphics::Gdi::GetDC;
use windows::Win32::Graphics::Gdi::GetDIBits;
use windows::Win32::Graphics::Gdi::GetObjectW;
use windows::Win32::Graphics::Gdi::ReleaseDC;
use windows::Win32::Graphics::Gdi::BITMAP;
use windows::Win32::Graphics::Gdi::BITMAPINFO;
use windows::Win32::Graphics::Gdi::BITMAPINFOHEADER;
use windows::Win32::Graphics::Gdi::BI_RGB;
use windows::Win32::Graphics::Gdi::DIB_RGB_COLORS;
use windows::Win32::Graphics::Gdi::HGDIOBJ;
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::UI::Shell::SHGetFileInfoW;
use windows::Win32::UI::Shell::SHFILEINFOW;
use windows::Win32::UI::Shell::SHGFI_ICON;
use windows::Win32::UI::WindowsAndMessaging::GetIconInfo;
use windows::Win32::UI::WindowsAndMessaging::HICON;

pub unsafe fn get_hicon(file_path: &str) -> Option<HICON> {
    let wide_path: Vec<u16> = OsStr::new(file_path).encode_wide().chain(Some(0)).collect();
    let mut shfileinfo: SHFILEINFOW = std::mem::zeroed();

    let result = SHGetFileInfoW(
        PCWSTR::from_raw(wide_path.as_ptr()),
        FILE_FLAGS_AND_ATTRIBUTES(0),
        Some(&mut shfileinfo as *mut SHFILEINFOW),
        std::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_ICON,
    );

    if result == 0 {
        None
    } else {
        Some(shfileinfo.hIcon)
    }
}

pub unsafe fn icon_to_image(icon: HICON) -> Result<RgbaImage, Box<dyn std::error::Error>> {
    let bitmap_size_i32 = std::mem::size_of::<BITMAP>() as i32;
    let biheader_size_u32 = std::mem::size_of::<BITMAPINFOHEADER>() as u32;

    let mut info = MaybeUninit::uninit();
    GetIconInfo(icon, info.as_mut_ptr())?;
    let info = info.assume_init();
    let _ = DeleteObject(info.hbmMask);

    let mut bitmap: MaybeUninit<BITMAP> = MaybeUninit::uninit();
    let _ = GetObjectW(
        HGDIOBJ(info.hbmColor.0),
        bitmap_size_i32,
        Some(bitmap.as_mut_ptr().cast()),
    );

    let bitmap = bitmap.assume_init();

    let width_u32 = bitmap.bmWidth as u32;
    let height_u32 = bitmap.bmHeight as u32;
    let width_usize = bitmap.bmWidth as usize;
    let height_usize = bitmap.bmHeight as usize;
    let buf_size = width_usize.checked_mul(height_usize).unwrap_or_default();
    let mut buf: Vec<u32> = Vec::with_capacity(buf_size);

    let dc = GetDC(HWND(ptr::null_mut()));

    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: biheader_size_u32,
            biWidth: bitmap.bmWidth,
            biHeight: -bitmap.bmHeight,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [Default::default()],
    };
    let _ = GetDIBits(
        dc,
        info.hbmColor,
        0,
        height_u32,
        Some(buf.as_mut_ptr().cast()),
        &mut bitmap_info,
        DIB_RGB_COLORS,
    );
    buf.set_len(buf.capacity());

    let _ = ReleaseDC(HWND(ptr::null_mut()), dc);
    let _ = DeleteObject(info.hbmColor);

    Ok(RgbaImage::from_fn(width_u32, height_u32, |x, y| {
        let x_usize = x as usize;
        let y_usize = y as usize;
        let idx = y_usize * width_usize + x_usize;
        let [b, g, r, a] = buf[idx].to_le_bytes();
        [r, g, b, a].into()
    }))
}




pub fn read_image_to_base64(file_path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(general_purpose::STANDARD.encode(&buffer))
}

pub fn get_icon_from_base64(base64: &str) -> Result<RgbaImage, Box<dyn Error>> {
    let buffer = general_purpose::STANDARD.decode(base64)?;
    let image = image::load_from_memory(&buffer)?;
    Ok(image.to_rgba8())
}