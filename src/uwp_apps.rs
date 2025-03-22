use std::error::Error;
use std::fs;
use std::path::Path;
use std::io::ErrorKind;
use std::path::PathBuf;
use image::RgbaImage;
use regex::Regex;
use crate::IconMatcher;
use crate::utils::image_utils::get_icon_from_base64;
use crate::utils::image_utils::read_image_to_base64;

pub fn get_uwp_icon(process_path: &str, icon_matcher: &IconMatcher) -> Result<RgbaImage, Box<dyn Error>> {
    let icon_path = &get_icon_file_path(process_path, icon_matcher)?;
    let base64 = read_image_to_base64(icon_path)?;
    let icon = get_icon_from_base64(&base64)?;
    Ok(icon)
}

pub fn get_uwp_icon_base64(process_path: &str, icon_matcher: &IconMatcher) -> Result<String, Box<dyn Error>> {
    let icon_path = get_icon_file_path(process_path, icon_matcher)?;
    let base64 = read_image_to_base64(&icon_path)?;
    Ok(base64)
}

fn get_icon_file_path(app_path: &str, icon_matcher: &IconMatcher) -> Result<String, Box<dyn Error>> {
    let package_folder = Path::new(app_path).parent().unwrap();

    let desktop_icon_path = package_folder.join("assets").join("DesktopShortcut.ico");

    if desktop_icon_path.exists() {
        Ok(desktop_icon_path.to_str().unwrap().to_string())
    } else {
        let manifest_path = package_folder.join("AppxManifest.xml");
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let icon_path = extract_icon_path(&manifest_content);
        if icon_path.is_none() {
            return Err(Box::new(std::io::Error::new(
                ErrorKind::NotFound,
                "Icon path not found in manifest"
            )));
        };
        let icon_path = icon_path.unwrap();
        let icon_full_path = package_folder.join(icon_path).to_str().unwrap().to_string();
        let icon_scale_path = match_icon_path(&icon_full_path, icon_matcher).unwrap_or(icon_full_path.to_string());

        Ok(icon_scale_path)
    }
}

fn extract_icon_path(manifest_content: &str) -> Option<String> {
    let doc = roxmltree::Document::parse(manifest_content).ok()?;

    let icon_path = doc
        .descendants()
        .find(|n| n.has_tag_name("Applications"))?
        .descendants()
        .find(|n|
            n.has_tag_name("Application") &&
                n.attributes().find(|a| a.name() == "Id" && a.value() == "App" ).is_some())?
        .descendants()
        .find(|n| n.has_tag_name("VisualElements"))?
        .attributes()
        .find(|a| a.name() == "Square44x44Logo")?
        .value();

    Some(icon_path.to_string())
}

fn file_stem_contains(path_buf: &PathBuf, text: &str) -> bool {
    path_buf.file_stem().unwrap_or_default().to_str().unwrap_or_default().contains(text)
}

fn match_icon_path(icon_path: &str, icon_matcher: &IconMatcher) -> Option<String> {
    let path = Path::new(icon_path);
    if path.is_dir() {
        return None
    }
    let folder_path = path.parent()?;
    let file_stem = path.file_stem()?.to_str()?;
    let mut matching_files: Vec<PathBuf> = folder_path.read_dir().ok()?
        .filter_map(|de| de.ok())
        .map(|de| de.path())
        .filter(|p| p.is_file())
        .filter(|p| file_stem_contains(p, file_stem))
        .collect();
    let scale = icon_matcher.display_scale;

    matching_files = reduce_to_best_scale(&matching_files, scale).unwrap_or(matching_files);

    if matching_files.is_empty() {
        return None
    }

    let first = matching_files[0].as_path();

    Some(first.to_str()?.to_string())
}

fn reduce_to_best_scale(matching_files: &Vec<PathBuf>, scale: i16) -> Option<Vec<PathBuf>> {
    let re = Regex::new("scale-(.*[0-9])").ok()?;
    let removal_candidates: Vec<&PathBuf> = matching_files
        .iter()
        .filter(|p| file_stem_contains(p,"scale-"))
        .collect();

    if removal_candidates.is_empty() {
        return None
    }

    let mut best_match = -1;

    for path_buf in removal_candidates.iter() {
        let stem = path_buf.file_stem()?.to_str()?;
        let captures = re.captures(stem);
        if captures.is_none() {
            continue
        }
        let current_match = captures?.get(1)?. as_str();
        let current_match = str::parse::<i16>(current_match).unwrap_or(best_match);
        best_match = current_match;
        if current_match == scale {
            best_match = current_match;
            break;
        }
        if current_match >= scale {
            best_match = current_match
        }
    }

    let mut paths_to_remove: Vec<&PathBuf> = removal_candidates
        .iter().filter(|p| !file_stem_contains(p,&format!("scale-{best_match}")))
        .map(|p| *p)
        .collect();

    if paths_to_remove.is_empty() {
        return None
    };

    let mut all_paths = matching_files.clone();

    for path in paths_to_remove {
        let i = all_paths.iter().position(|p| p == path);
        match i {
            None => { continue }
            Some(i) => { all_paths.remove(i); }
        }
    }

    Some(all_paths)
}