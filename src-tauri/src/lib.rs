use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};
use tauri::{path::BaseDirectory, Manager};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::UI::Shell::CommandLineToArgvW;
use windows_icons::{get_icon_by_path_with_size, IconSize};

#[derive(Debug, Serialize, Deserialize)]
struct ApplicationRecord {
    id: String,
    name: String,
    executable: String,
    logo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApplicationEntry {
    name: String,
    executable: String,
}

#[derive(Debug, Deserialize)]
struct ConfigRoot {
    applications: HashMap<String, ApplicationEntry>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let exe_path = std::env::current_exe()?;
            let images_dir = exe_path
                .parent()
                .ok_or_else(|| format!("Failed to determine application directory"))?
                .to_path_buf()
                .join("images");
            Ok(app
                .asset_protocol_scope()
                .allow_directory(images_dir, true)?)
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![read_config, open_app])
        .run(tauri::generate_context!())
        .expect("error while running application");
}

// Reads the configuration from the bundled config.yaml file and returns a list of application records.
#[tauri::command]
fn read_config(app: tauri::AppHandle) -> Result<Vec<ApplicationRecord>, String> {
    let config_path = app
        .path()
        .resolve("config.yaml", BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve bundled config path: {e}"))?;

    let config_content: String = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file at {}: {e}", config_path.display()))?;

    println!("Config content:\n{}", config_content);

    let root: ConfigRoot =
        serde_yaml::from_str(&config_content).map_err(|e| format!("Failed to parse YAML: {e}"))?;

    let apps: Vec<ApplicationRecord> = root
        .applications
        .into_iter()
        .map(|(id, entry)| ApplicationRecord {
            id,
            name: entry.name,
            executable: entry.executable.clone(),
            logo: cache_icon(&entry.executable)
                .ok()
                .map(|p| p.to_string_lossy().to_string()),
        })
        .collect::<Vec<_>>();
    Ok(apps)
}

#[cfg(windows)]
fn cache_icon(path: &str) -> Result<PathBuf, Box<dyn Error>> {
    let exe_path = std::env::current_exe()?;

    let images_dir = exe_path
        .parent()
        .ok_or_else(|| format!("Failed to determine application directory"))?
        .to_path_buf()
        .join("images");
    fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {e}"))?;

    // Generate hasher for the executable path to create a unique cache file name.
    let app_path = extract_app_path(&path);

    if let Some(app_path) = app_path {
        let logo_hash = hash_path(app_path.as_path());
        let icon = get_icon_by_path_with_size(app_path, IconSize::ExtraLarge).map_err(|e| e.to_string())?;

        let icon_path = images_dir.join(format!("{logo_hash}.png"));
        icon.save(&icon_path)
            .map_err(|e| format!("Failed to save icon: {e}"))?;
        return Ok(icon_path);
    } else {
        return Err(format!("Failed to extract executable path from: {}", path).into());
    }
}

// Extracts the full path of the executable from a command line string.
#[cfg(windows)]
fn extract_app_path(path: &str) -> Option<PathBuf> {
    let trimmed = path.trim();
    let striped = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed);
    let lower = striped.to_ascii_lowercase();

    let end = [".exe", ".lnk"]
        .iter()
        .filter_map(|ext| lower.find(ext).map(|i| i + ext.len()))
        .min()?;

    let parse_input = if !striped.starts_with('"')
    {
        let striped = &striped[..end];
        format!("\"{striped}\"")
    } else {
        striped.to_string()
    };

    let wide_path: Vec<u16> = parse_input.encode_utf16().chain(Some(0)).collect();
    // let wide_path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    let mut argc = 0;

    let argv = unsafe { CommandLineToArgvW(PCWSTR(wide_path.as_ptr()), &mut argc) };

    let result = unsafe {
        use std::os::windows::ffi::OsStringExt;

        if argv.is_null() || argc == 0 {
            return None;
        } else {
            let first_element = *argv;
            let mut len = 0;

            while *first_element.0.add(len) != 0 {
                len += 1;
            }

            let exe = OsString::from_wide(std::slice::from_raw_parts(first_element.0, len));
            let exe_path = PathBuf::from(exe);

            if exe_path.is_absolute()
                && exe_path.is_file()
                && exe_path
                    .extension()
                    .is_some_and(|s| s.eq_ignore_ascii_case("exe") || s.eq_ignore_ascii_case("lnk"))
            {
                Some(exe_path)
            } else {
                None
            }
        }
    };

    unsafe {
        LocalFree(Some(HLOCAL(argv as *mut _)));
    }

    result
}

// Generates a short hash for a given path string.
fn hash_path(path: &Path) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(path.as_os_str().to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest)[..16].to_string()
}

// Opens the specified application using the default system handler.
#[tauri::command]
fn open_app(path: &Path) {
    let _ = open::that(path).map_err(|e| format!("Failed to open executable: {e}"));
}
