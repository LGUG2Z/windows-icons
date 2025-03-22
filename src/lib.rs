use base64::Engine as _;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use base64::engine::general_purpose;
use image::RgbaImage;
use crate::utils::image_utils::get_hicon;
use crate::utils::image_utils::icon_to_image;
use crate::utils::process_utils::get_process_id_by_hwnd;
use crate::utils::process_utils::get_process_path;
use crate::uwp_apps::get_uwp_icon;
use crate::uwp_apps::get_uwp_icon_base64;

mod utils {
    pub mod image_utils;
    pub mod process_utils;
}

mod uwp_apps;
pub struct IconMatcher {
    pub display_scale: i16,
}

impl Default for IconMatcher {
    fn default() -> Self {
        IconMatcher {
            display_scale: 100
        }
    }
}

enum AppType {
    UWP,
    Other
}

fn get_app_type(path: &str) -> AppType {
    if path.contains("WindowsApps") {
        return AppType::UWP
    }

    AppType::Other
}

pub fn get_icon_by_hwnd(hwnd: isize) -> Option<RgbaImage> {
    let icon_matcher = &IconMatcher::default();
    get_icon_by_hwnd_matching(hwnd, icon_matcher)
}

pub fn get_icon_by_hwnd_matching(hwnd: isize, icon_matcher: &IconMatcher) -> Option<RgbaImage> {
    let pid = get_process_id_by_hwnd(hwnd)?;
    let icon = get_icon_by_process_id_matching(pid, icon_matcher);
    icon
}

pub fn get_icon_base64_by_hwnd_matching(hwnd: isize, icon_matcher: &IconMatcher) -> Option<String> {
    let pid = get_process_id_by_hwnd(hwnd)?;
    let icon = get_icon_base64_by_process_id_matching(pid, icon_matcher);
    icon
}

pub fn get_icon_by_process_id(process_id: u32) -> Option<RgbaImage> {
    let icon_matcher = &IconMatcher::default();
    get_icon_by_process_id_matching(process_id, icon_matcher)
}

pub fn get_icon_by_process_id_matching(process_id: u32, icon_matcher: &IconMatcher) -> Option<RgbaImage> {
    let path = &get_process_path(process_id)?;
    match get_app_type(path) {
        AppType::UWP => { get_uwp_icon(path, icon_matcher).ok() }
        AppType::Other => {get_icon_by_path(path)}
    }
}

pub fn get_icon_by_path(path: &str) -> Option<RgbaImage> {
    unsafe {
        get_hicon(path).map(|icon| icon_to_image(icon).ok())?
    }
}

pub fn get_icon_base64_by_process_id(process_id: u32) -> Option<String> {
    let icon_matcher = &IconMatcher::default();
    let path = get_process_path(process_id)?;
    get_icon_base64_by_path_matching(&path, icon_matcher)
}

pub fn get_icon_base64_by_process_id_matching(process_id: u32, icon_matcher: &IconMatcher) -> Option<String> {
    let path = get_process_path(process_id)?;
    get_icon_base64_by_path_matching(&path, icon_matcher)
}

pub fn get_icon_base64_by_path(path: &str) -> Option<String>{
    let icon_matcher = &IconMatcher::default();
    get_icon_base64_by_path_matching(path, icon_matcher)
}

pub fn get_icon_base64_by_path_matching(path: &str, icon_matcher: &IconMatcher) -> Option<String> {
    match get_app_type(path) {
        AppType::UWP => {get_uwp_icon_base64(path, icon_matcher).ok()}
        AppType::Other => {
            if let Some(icon_image) = get_icon_by_path(path) {
                let mut buffer = Vec::new();
                icon_image
                    .write_to(
                        &mut std::io::Cursor::new(&mut buffer),
                        image::ImageFormat::Png,
                    ).ok();
                Some(general_purpose::STANDARD.encode(buffer))
            } else {
                None
            }
        }
    }
}



