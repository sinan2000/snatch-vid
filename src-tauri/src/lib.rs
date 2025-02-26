// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::command;
use tokio::task;

// Struct for storing the file path
#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    dir: String,
}

// Get the download directory from the config file
fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("config_file.json")
}

#[command]
fn config_exists() -> bool {
    get_config_path().exists()
}

// Create and save a new config at the path specified
#[command]
fn create_config(dir: String) {
    let config = AppConfig { dir };
    let path = get_config_path();

    if let Ok(config_json) = serde_json::to_string_pretty(&config) {
        let _ = fs::write(path, config_json);
    }
}

// Read the config returning the `dir` field
#[command]
fn read_config() -> Option<String> {
    let path = get_config_path();

    if let Ok(contents) = fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str::<AppConfig>(&contents) {
            return Some(config.dir);
        }
    }

    None // if it doesn't exist
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

// Struct for parsing yt-dlp JSON response
#[derive(Debug, Deserialize)]
struct YtDlpJSON {
    #[serde(rename = "_type")]
    entry_type: Option<String>,
    entries: Option<Vec<serde_json::Value>>,
    id: Option<String>,
}

// Detect the type of URL and return 'video', 'playlist', or 'none'
#[command]
async fn detect_url_type(url: String) -> String {
    let (yt_dlp_bin, _) = get_binary_paths();

    let result = task::spawn_blocking(move || {
        let output = Command::new(yt_dlp_bin)
            .arg("--dump-json")
            .arg("--no-warnings")
            .arg(url)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    let json_output = String::from_utf8_lossy(&result.stdout);

                    // Debugging: Print response
                    println!("yt-dlp Full Response: {}", json_output);

                    // Try parsing JSON
                    match serde_json::from_str::<YtDlpJSON>(&json_output) {
                        Ok(parsed) => {
                            // 1️. Check if `_type == "playlist"`
                            if let Some(ref t) = parsed.entry_type {
                                if t == "playlist" {
                                    return "playlist".to_string();
                                }
                            }
                            // 2️. Check if `entries` field is present and non-empty
                            if let Some(entries) = parsed.entries {
                                if !entries.is_empty() {
                                    return "playlist".to_string();
                                }
                            }
                            // 3️. If it has an `id` but no `entries`, it's a video
                            if parsed.id.is_some() {
                                return "video".to_string();
                            }
                            // 4️. Fallback: No valid data, return "none"
                            return "none".to_string();
                        }
                        Err(_) => "none".to_string(), // JSON parsing error
                    }
                } else {
                    "none".to_string() // Command ran but failed
                }
            }
            Err(_) => "none".to_string(), // Execution failed
        }
    })
    .await
    .unwrap_or_else(|_| "none".to_string());

    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            config_exists,
            create_config,
            read_config,
            detect_url_type
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
