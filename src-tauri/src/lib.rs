use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, FilePath};

// Struct for storing the file path
#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    download_dir: Option<String>,
}

// Get the download directory from the config file
fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("snatchvid_config.json")
}

// Load the config file or create if doesn't exist
fn load_or_create_config() -> AppConfig {
    let config_path = get_config_path();

    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<AppConfig>(&contents) {
            return config;
        }
    }

    // Return a new config if the file doesn't exist or is invalid
    AppConfig { download_dir: None }
}

// Save updated config
fn save_config(config: &AppConfig) {
    let config_path = get_config_path();
    if let Ok(config_json) = serde_json::to_string_pretty(config) {
        let _ = fs::write(config_path, config_json);
    }
}

// Asks user to choose a directory for storing saved downloads
fn prompt_directory(app_handle: &AppHandle) -> Option<String> {
    let (sender, receiver) = std::sync::mpsc::channel();

    app_handle.dialog().file().pick_folder(move |folder_path: Option<FilePath>| {
        if let Some(file_path) = folder_path {
            if let Some(path) = file_path.as_path() { 
                sender
                    .send(path.display().to_string())
                    .unwrap();
            }
        } else {
            sender.send(String::new()).unwrap(); // Send empty string if canceled
        }
    });

    receiver.recv().ok().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    })
}


// Ensure folder exists; ask user to choose if necessary
fn ensure_download_directory(app_handle: &AppHandle) -> String {
    let mut config = load_or_create_config();

    if let Some(ref dir) = config.download_dir {
        let path = Path::new(dir);
        if path.exists() {
            return dir.clone();
        }
    }

    if let Some(new_dir) = prompt_directory(app_handle) {
        config.download_dir = Some(new_dir.clone());
        save_config(&config);
        return new_dir;
    }

    panic!("Download folder must be selected!");
}

// Detect correct binary paths for `yt-dlp` and `ffmpeg`
fn get_binary_paths() -> (String, String) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let yt_dlp_bin = match (os, arch) {
        ("windows", "x86_64") => "bin/yt-dlp.exe",
        ("windows", "x86") => "bin/yt-dlp_x86.exe",
        ("macos", "aarch64") => "bin/yt-dlp_macos",
        ("macos", "x86_64") => "bin/yt-dlp_macos",
        _ => panic!("Unsupported OS or architecture!"),
    };

    let ffmpeg_bin = match (os, arch) {
        ("windows", "x86_64") => "bin/ffmpeg.exe",
        ("windows", "x86") => "bin/ffmpeg_x86.exe",
        ("macos", "aarch64") => "bin/ffmpeg_macos_arm",
        ("macos", "x86_64") => "bin/ffmpeg_macos_x86",
        _ => panic!("Unsupported OS or architecture!"),
    };

    (yt_dlp_bin.to_string(), ffmpeg_bin.to_string())
}

/// Tauri command to return the binary paths
#[tauri::command]
fn get_binaries() -> (String, String) {
    get_binary_paths()
}

/// Tauri command to return the download directory
#[tauri::command]
fn get_download_directory() -> String {
    let config = load_or_create_config();
    config.download_dir.unwrap_or_default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_binaries,
            get_download_directory
        ])
        .setup(|app| {
            let _ = ensure_download_directory(&app.handle()); // Run this at startup
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the application");
}
