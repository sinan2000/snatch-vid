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

// Struct for parsing yt-dlp JSON output
#[derive(Debug, Deserialize)]
struct YtDlpJSON {
    #[serde(rename = "_type")]
    entry_type: Option<String>,
    entries: Option<Vec<serde_json::Value>>,
    id: Option<String>,
    playlist_count: Option<u32>,
}

// Detect the type of URL passed (video, playlist, or none)
#[command]
async fn detect_url_type(url: String) -> String {
    let (yt_dlp_bin, _) = get_binary_paths();
    task::spawn_blocking(move || {
        let output = Command::new(yt_dlp_bin)
            .arg("-J")
            .arg("--no-warnings")
            .arg(url)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let json_output = String::from_utf8_lossy(&output.stdout);
        let parsed: YtDlpJSON = serde_json::from_str(&json_output).ok()?;
        if let Some(ref t) = parsed.entry_type {
            if t == "playlist" {
                if parsed.playlist_count.unwrap_or(1) == 0
                    || parsed
                        .entries
                        .as_ref()
                        .map_or(false, |entries| entries.is_empty())
                {
                    return Some("none".to_string());
                }
                return Some("playlist".to_string());
            }
        }
        if parsed.id.is_some() {
            Some("video".to_string())
        } else {
            Some("none".to_string())
        }
    })
    .await
    .unwrap_or(None)
    .unwrap_or_else(|| "none".to_string())
}

// Fetches the playlist title of the given URL
fn get_playlist_title(yt_dlp_path: &str, url: &str) -> Result<String, String> {
    let output = Command::new(yt_dlp_path)
        .args(&["--print", "%(playlist_title)s"]) // Removed --flat-playlist
        .arg(url)
        .output()
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        return Err("yt-dlp failed to fetch playlist title.".to_string());
    }

    let title = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .replace("\n", "_");

    if title.is_empty() {
        return Err("Playlist title is empty. Check URL.".to_string());
    }
    Ok(title)
}

// Creates a new directory at the given path
fn create_folder(base_dir: &str, title: &str) -> bool {
    let mut folder_path = PathBuf::from(base_dir);
    folder_path.push(title);

    let mut count = 2;
    while folder_path.exists() {
        folder_path = PathBuf::from(base_dir);
        folder_path.push(format!("{} ({})", title, count));
        count += 1;
    }

    fs::create_dir_all(&folder_path).is_ok()
}

// Creates a new folder where the playlist will be downloaded
#[command]
async fn setup_playlist_folder(url: String) -> bool {
    let result = task::spawn_blocking(move || {
        let (yt_dlp_path, _) = get_binary_paths();

        let base_dir = match read_config() {
            Some(dir) => dir,
            None => return false,
        };

        let title = match get_playlist_title(&yt_dlp_path, &url) {
            Ok(t) => t,
            Err(_) => return false,
        };

        create_folder(&base_dir, &title)
    })
    .await
    .unwrap_or(false);

    result
}

// Generates the arguments for the yt-dlp command
fn generate_args(
    format: &str,
    quality: &str,
    download_type: &str,
    ffmpeg_path: &str,
) -> Vec<String> {
    let mut args = vec![
        "--ffmpeg-location".to_string(),
        ffmpeg_path.to_string(),
        "--verbose".to_string(),
    ];

    if matches!(format, "mp3" | "wav" | "aac" | "flac") {
        args.push("-x".to_string());
        args.push("--audio-format".to_string());
        args.push(format.to_string());
        args.push("--audio-quality".to_string());
        args.push("0".to_string());
    } else {
        let quality_map = match quality {
            "4k" => "bestvideo[height=2160]+bestaudio[]/best[height=2160]",
            "1440p" => "bestvideo[height=1440]+bestaudio/best[height=1440]",
            "1080p" => "bestvideo[height=1080]+bestaudio/best[height=1080]",
            "720p" => "bestvideo[height=720]+bestaudio/best[height=720]",
            "480p" => "bestvideo[height=480]+bestaudio/best[height=480]",
            "360p" => "bestvideo[height=360]+bestaudio/best[height=360]",
            "240p" => "bestvideo[height=240]+bestaudio/best[height=240]",
            "144p" => "bestvideo[height=144]+bestaudio/best[height=144]",
            _ => "bestvideo+bestaudio/best", // Default to best quality
        };
        args.push("-f".to_string());
        args.push(quality_map.to_string());
        args.push("--merge-output-format".to_string());
        args.push(format.to_string());
    }

    args
}

// Starts the download process
#[command]
async fn start_download(
    url: String,
    format: String,
    quality: String,
    downloadType: String,
) -> bool {
    let (yt_dlp_path, ffmpeg_path) = get_binary_paths();

    let args = generate_args(&format, &quality, &downloadType, &ffmpeg_path);

    let result = task::spawn_blocking(move || {
        if downloadType == "playlist" {
            download_playlist(&yt_dlp_path, &url, args)
        } else {
            download_video(&yt_dlp_path, &url, args)
        }
    })
    .await
    .unwrap_or(false);

    result
}

fn download_video(yt_dlp_path: &str, url: &str, args: Vec<String>) -> bool {
    let output = Command::new(yt_dlp_path).arg(url).args(&args).output();

    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

fn download_playlist(yt_dlp_path: &str, url: &str, args: Vec<String>) -> bool {
    let base_dir = match read_config() {
        Some(dir) => dir,
        None => return false,
    };

    let title = match get_playlist_title(&yt_dlp_path, &url) {
        Ok(t) => t,
        Err(_) => return false,
    };

    let mut folder_path = PathBuf::from(&base_dir).join(&title);
    let mut counter = 2;
    while folder_path.exists() {
        folder_path = PathBuf::from(&base_dir).join(format!("{}_{}", title, counter));
        counter += 1;
    }

    if fs::create_dir_all(&folder_path).is_err() {
        return false;
    }

    let mut modified_args = args.clone();
    modified_args.push("-P".to_string());
    modified_args.push(folder_path.to_string_lossy().to_string());

    // Run yt-dlp
    let output = Command::new(yt_dlp_path)
        .arg(url)
        .args(modified_args)
        .output();

    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
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
            detect_url_type,
            setup_playlist_folder,
            start_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
