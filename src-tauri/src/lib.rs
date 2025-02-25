// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::fs;
use std::sync::Arc;
use tauri::{Window, Manager};
use tauri::Event;
use tauri::Runtime;
use tauri::Emitter;
use tauri_plugin_opener;

// Returns binary paths abstracting OS differences
fn get_binary_paths() -> (String, String) {
    let yt_dlp_path = if cfg!(target_os = "windows") {
        "./bin/yt-dlp.exe".to_string()
    } else {
        "./bin/yt-dlp_macos".to_string()
    };

    let ffmpeg_path = if cfg!(target_os = "windows") {
        "./bin/ffmpeg.exe".to_string()
    } else {
        "./usr/local/bin/ffmpeg".to_string()
    };

    (yt_dlp_path, ffmpeg_path)
}

// Checks if FFmpeg is installed on macOS; if not, tries to install it
fn check_ffmpeg() -> bool {
    if cfg!(target_os = "macos") {
        let check_ffmpeg = Command::new("which")
            .arg("ffmpeg")
            .output()
            .expect("Failed to check FFmpeg");

        if check_ffmpeg.status.success() {
            return true;
        }

        println!("Installing FFmpeg via Homebrew...");
        let install_ffmpeg = Command::new("brew")
            .arg("install")
            .arg("ffmpeg")
            .status()
            .expect("Failed to install FFmpeg");

        return install_ffmpeg.success();
    }

    true // for windows
}

fn get_format_code(format: &str, quality: &str, ffmpeg_available: bool) -> String {
    let quality_map = match quality {
        "4k" => "bestvideo[height=2160]+bestaudio[ext=m4a]/best",
        "1440p" => "bestvideo[height=1440]+bestaudio[ext=m4a]/best",
        "1080p" => "bestvideo[height=1080]+bestaudio[ext=m4a]/best",
        "720p" => "bestvideo[height=720]+bestaudio[ext=m4a]/best",
        "480p" => "bestvideo[height=480]+bestaudio[ext=m4a]/best",
        "360p" => "bestvideo[height=360]+bestaudio[ext=m4a]/best",
        "240p" => "bestvideo[height=240]+bestaudio[ext=m4a]/best",
        "144p" => "bestvideo[height=144]+bestaudio[ext=m4a]/best",
        _ => "bestvideo+bestaudio[ext=m4a]/best", // Default to best quality
    };

    match format {
        "mp3" => "bestaudio[ext=m4a]/bestaudio".to_string(),
        "wav" => "bestaudio[ext=wav]/bestaudio".to_string(),
        "aac" => "bestaudio[ext=aac]/bestaudio".to_string(),
        "flac" => "bestaudio[ext=flac]/bestaudio".to_string(),
        _ => {
            if ffmpeg_available {
                quality_map.to_string()
            } else {
                // fallback for avoiding to merge video and audio
                "best[ext=mp4]/best".to_string()
            }
        }
    }
}

// Fetch the title for storing the playlist videos in a folder
fn get_playlist_title(yt_dlp_path: &str, url: &str) -> Result<String, String> {
    let output = Command::new(yt_dlp_path)
    .args(&["--flat-playlist", "--print", "%(playlist_title)s"])
    .arg(url)
    .output()
    .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        return Err("yt-dlp failed to fetch playlist title.".to_string());
    }
    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let clean_title = title.lines().next().unwrap_or("").trim().replace("\n", "_");
    if clean_title.is_empty() {
        return Err("Playlist title is empty. Check URL.".to_string());
    }
    Ok(clean_title)
}


// Builds the args and runs the download command
fn build_download_command(
    window: Window,
    yt_dlp_path: &str,
    ffmpeg_path: Option<&String>,
    url: &str,
    output_path: &str,
    format_code: &str,
    is_playlist: bool,
    start_index: Option<u32>,
) -> Result <String, String> {
    let mut command = Command::new(yt_dlp_path);
    command.arg("-o").arg(format!("{}/%(title)s.%(ext)s", output_path));
    command.arg("--progress-template").arg("%(progress._percent_str)s");

    if is_playlist {
        command.arg("--yes-playlist");
        if let Some(index) = start_index {
            command.arg("--playlist-start").arg(index.to_string());
        }
    } else {
        command.arg("--no-playlist");
    }


    if let Some (ffmpeg) = ffmpeg_path {
        command.arg("--ffmpeg-location").arg(ffmpeg);
    }

    command
        .arg("--merge-output-format")
        .arg("mp4")
        .arg("--format")
        .arg(format_code)
        .arg("--verbose")
        .arg(url)
        .stdout(Stdio::piped());

    let mut child = command.spawn().map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;
    let stdout = child.stdout.take().unwrap();

    let window = Arc::new(window);
    let reader = BufReader::new(stdout);
    let window_clone = Arc::clone(&window);

    std::thread::spawn(move || {
        for line in reader.lines() {
            if let Ok(log) = line {
                if let Some(percentage) = log.split_whitespace().find(|s| s.contains('%')) {
                    let _ = window_clone.emit("download_progress", percentage.trim());
                }
            }
        }
    });

    let status = child.wait().map_err(|e| format!("Failed to wait for yt-dlp: {}", e))?;
    
    if status.success() {
        let _ = window.emit("download_complete", "Finished!");
        Ok("Finished".to_string())
    } else {
        let _ = window.emit("download_failed", "Failed!");
        Err("Failed to download".to_string())
    }
}

// Downloads a video from a given URL
#[tauri::command]
fn download_video(
    window: Window,
    url: String,
    output_path: String,
    format: String,
    quality: String
) -> Result<String, String> {
    let (yt_dlp_path, ffmpeg_path) = get_binary_paths();
    let ffmpeg_available = check_ffmpeg();
    let format_code = get_format_code(&format, &quality, ffmpeg_available);

    let result = build_download_command(
        window.clone(),
        &yt_dlp_path,
        Some(&ffmpeg_path),
        &url,
        &output_path,
        &format_code,
        false,
        None
    );

    match result {
        Ok(_) => {
            let _ = window.emit("download_complete", "Finished! ✅");
        }
        Err(_) => {
            let _ = window.emit("download_failed", "Failed ❌");
        }
    }

    Ok("Downloading...".to_string())
}

// Downloads a full playlist
#[tauri::command]
fn download_playlist(
    window: Window,
    url: String,
    output_path: String,
    format: String,
    quality: String,
    start_index: Option<u32>,
) -> Result<String, String> {
    let (yt_dlp_path, ffmpeg_path) = get_binary_paths();
    let ffmpeg_available = check_ffmpeg();
    let format_code = get_format_code(&format, &quality, ffmpeg_available);

    let playlist_title = match get_playlist_title(&yt_dlp_path, &url) {
        Ok(title) => title,
        Err(e) => return Err(e),
    };

    let playlist_path = format!("{}/{}", output_path, playlist_title);

    if let Err(e) = fs::create_dir_all(&playlist_path) {
        return Err(format!("Failed to create playlist folder: {}", e));
    }

    println!("Downloading playlist to: {}", playlist_path);

    let window_clone = window.clone();
    std::thread::spawn(move || {
        let _ = build_download_command(
            window_clone,
            &yt_dlp_path,
            Some(&ffmpeg_path),
            &url,
            &playlist_path,
            &format_code,
            true,
            start_index
        );
    });

    Ok("Downloading...".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![download_video, download_playlist])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
