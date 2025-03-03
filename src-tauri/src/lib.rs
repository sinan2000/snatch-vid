// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{env, fs};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::command;
use tauri::Emitter;
use tokio::task;
use std::os::windows::process::CommandExt;

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

// Detect the type of URL passed (video, playlist, or none) and return it with playlist title if present
#[command]
async fn detect_url_type(url: String) -> (String, String) {
    let (yt_dlp_bin, _) = get_binary_paths();

    task::spawn_blocking(move || {
        #[cfg(windows)]
        let output = Command::new(env::current_dir().unwrap().join(&yt_dlp_bin))
            .arg("-J")
            .arg("--no-warnings")
            .arg("--flat-playlist")
            .arg(&url)
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .ok()?;

        #[cfg(not(windows))]
        let output = Command::new(yt_dlp_bin)
            .arg("-J")
            .arg("--no-warnings")
            .arg("--flat-playlist")
            .arg(url)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json_output = String::from_utf8_lossy(&output.stdout);

        let parsed: Value = match serde_json::from_str(&json_output) {
            Ok(parsed) => parsed,
            Err(err) => {
                eprintln!("Failed to parse yt-dlp JSON output: {}", err);
                return None;
            }
        };

        if parsed.get("_type").map_or(false, |t| t == "playlist") {
            if let Some(entries) = parsed.get("entries").and_then(|e| e.as_array()) {
                if !entries.is_empty() {
                    let title = parsed.get("title").and_then(|t| t.as_str()).unwrap_or("none").to_string();
                    return Some(("playlist".to_string(), title));
                }
            }
        }

        if parsed.get("_type").map_or(false, |t| t == "video") {
            return Some(("video".to_string(), "none".to_string()));
        }

        None
    })
    .await
    .unwrap_or(None)
    .unwrap_or_else(|| ("none".to_string(), "none".to_string()))
}

// Creates a new directory at the given path
fn create_folder(base_dir: &str, title: &str) -> Option<String> {
    let mut folder_path = PathBuf::from(base_dir);
    folder_path.push(title);

    let mut count = 2;
    while folder_path.exists() {
        folder_path = PathBuf::from(base_dir);
        folder_path.push(format!("{} ({})", title, count));
        count += 1;
    }

    if fs::create_dir_all(&folder_path).is_ok() {
        return Some(folder_path.file_name()?.to_string_lossy().to_string());
    }

    None
}

// Creates a new folder where the playlist will be downloaded
#[command]
async fn setup_playlist_folder(title: String) -> Option<String> {
    let result = task::spawn_blocking(move || {
        let base_dir = match read_config() {
            Some(dir) => dir,
            None => return None,
        };
        create_folder(&base_dir, &title)
    })
    .await
    .unwrap_or(None);

    result
}

// Generates the arguments for the yt-dlp command
fn generate_args(
    format: &str,
    quality: &str,
    download_type: &str,
    ffmpeg_path: &str,
    download_path: &str,
) -> Vec<String> {
    let mut args = vec![
        "--ffmpeg-location".to_string(),
        ffmpeg_path.to_string(),
        "-P".to_string(),
        download_path.to_string(),
    ];

    match format {
        "mp4" => {
            args.push(format!(
                "-f bestvideo[height={}]+bestaudio[ext=m4a]/best",
                quality
            ));
            args.push("--merge-output-format".to_string());
            args.push("mp4".to_string());
        }
        "webm" => {
            args.push(format!(
                "-f bestvideo[height={}][ext=webm]+bestaudio[ext=webm]/best",
                quality
            ));
            args.push("--merge-output-format".to_string());
            args.push("webm".to_string());
        }
        "mp3" => {
            args.push("-f bestaudio".to_string());
            args.push("--extract-audio".to_string());
            args.push("--audio-format".to_string());
            args.push("mp3".to_string());
        }
        "m4a" => {
            args.push("-f bestaudio[ext=m4a]".to_string());
            args.push("--extract-audio".to_string());
            args.push("--audio-format".to_string());
            args.push("m4a".to_string());
        }
        "wav" => {
            args.push("-f bestaudio".to_string());
            args.push("--extract-audio".to_string());
            args.push("--audio-format".to_string());
            args.push("wav".to_string());
        }
        _ => {
            eprintln!("Invalid format provided: {}", format);
        }
    }

    if download_type == "playlist" {
        args.push("--yes-playlist".to_string());
    }

    // naming - title.extension
    args.push("-o".to_string());
    args.push("%(title)s.%(ext)s".to_string());

    args.push("--newline".to_string());

    args
}

// Starts the download process
#[command]
async fn start_download(
    window: tauri::Window,
    url: String,
    format: String,
    quality: String,
    download_type: String,
    playlist_folder: String,
) -> bool {
    let (yt_dlp_path, ffmpeg_path) = get_binary_paths();

    let download_path = match read_config() {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("Download path not found in config.");
            return false;
        }
    };

    let final_download_path = if download_type == "playlist" {
        download_path.join(&playlist_folder)
    } else {
        download_path
    };

    let path_str = final_download_path.to_string_lossy().to_string();

    let args = generate_args(&format, &quality, &download_type, &ffmpeg_path, &path_str);

    let result = task::spawn_blocking(move || download_process(&yt_dlp_path, &url, args, window))
        .await
        .unwrap_or(false);

    result
}

fn download_process(
    yt_dlp_path: &str,
    url: &str,
    args: Vec<String>,
    window: tauri::Window,
) -> bool {
    // On Windows, set the flag to not create a window.
    #[cfg(windows)]
    let mut child = {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new(env::current_dir().unwrap().join(yt_dlp_path))
            .arg(url)
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| {
                eprintln!("Failed to spawn process: {}", e);
                panic!();
            })
    };

    // For non-Windows systems, spawn normally.
    #[cfg(not(windows))]
    let mut child = {
        Command::new(yt_dlp_path)
            .arg(url)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| {
                eprintln!("Failed to spawn process: {}", e);
                panic!();
            })
    };

    // Take stdout and stderr and spawn threads to read them.
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let window_stdout = window.clone();
    let stdout_handle = std::thread::spawn(move || {
        if let Some(stdout) = stdout {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = window_stdout.emit("progress", line);
                }
            }
        }
    });

    let window_stderr = window.clone();
    let stderr_handle = std::thread::spawn(move || {
        if let Some(stderr) = stderr {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = window_stderr.emit("progress", line);
                }
            }
        }
    });

    let status = match child.wait() {
        Ok(status) => status,
        Err(e) => {
            eprintln!("Failed to wait for yt-dlp process: {}", e);
            return false;
        }
    };

    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    status.success()
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
