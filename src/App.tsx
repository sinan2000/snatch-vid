import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { desktopDir } from "@tauri-apps/api/path";
import { listen } from "@tauri-apps/api/event";
import { Logo } from "./logo";
import { LoadingModal } from "./loading";
import './App.css';

function App() {
  const [isPlaylist, setIsPlaylist] = useState<boolean>(false);
  const [videoUrl, setVideoUrl] = useState<string>("");
  const [downloadPath, setDownloadPath] = useState<string>(""); // set to current path
  const [startIndex, setStartIndex] = useState<number>(1);
  const [format, setFormat] = useState<string>("mp4");
  const [quality, setQuality] = useState<string>("1080p");
  const [logs, setLogs] = useState<string[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [finished, setFinished] = useState<boolean>(false);
  const [progress, setProgress] = useState<number>(0);

  useEffect(() => {
    async function fetchPath() {
      try {
        const defaultPath = await desktopDir();
        setDownloadPath(defaultPath);
      } catch (error) {
        console.error(error);
      }
    }

    fetchPath();
  }, []);

  useEffect(() => {
    const unlistenProgress = listen("download_progress", (event) => {
      const perc = parseFloat(event.payload as string);

      if(!isNaN(perc)) {
        setProgress(perc);
      }
    });

    const unlistenComplete = listen("download_complete", () => {
      setFinished(true);
    });

    const unlistenFailed = listen("download_failed", () => {
      setFinished(true);
    });

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
      unlistenFailed.then((fn) => fn());
    };
  }, []);

  async function selectFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
    });

    if (selected) setDownloadPath(selected as string);
  }

  async function handleDownload() {
    if (!videoUrl.trim()) {
      return;
    }

    setLoading(true);
    setFinished(false);
    setProgress(0);

    try {
      if (isPlaylist) {
        console.log("Downloading playlist...", videoUrl, downloadPath, format, quality, startIndex);
        await invoke("download_playlist", {
          url: videoUrl,
          outputPath: downloadPath,
          format,
          quality,
          startIndex: startIndex,
        });
      } else {
        await invoke("download_video", {
          url: videoUrl,
          outputPath: downloadPath,
          format,
          quality,
        });
      }

      setLogs((prevLogs) => [...prevLogs, `Downloaded: ${videoUrl}`]);
    } catch (error) {
      const errorMessage = `Error: ${error}`;
      setLogs((prevLogs) => [...prevLogs, errorMessage]);
      setFinished(true);
    }
  }

  return (
    <main className="flex flex-col items-center justify-center min-h-screen w-full bg-black text-white">
      <div className="w-full p-5">

        {/* Top Section */}
        <div className="flex justify-between items-center">
          <h1 className="text-2xl font-semibold">Speedyt</h1>
          <a href="https://www.snsautomation.tech" target="_blank" className="group flex items-center gap-2 transition-transform hover:scale-105">
            <Logo />
            <div className="flex flex-col">
              <span className="text-sm font-semibold text-white group-hover:text-emerald-400 transition-colors">
                SNS Automation
              </span>
              <span className="text-xs text-emerald-500/60">Software Solutions</span>
            </div>
          </a>
        </div>

        {/* Divider */}
        <div className="w-full max-w-3xl my-8 border-t border-gray-600"></div>

        {/* Video / Playlist Selection */}
        <div className="mt-4">
          <label className="mr-4">
            <input
              type="radio"
              name="type"
              value="video"
              checked={!isPlaylist}
              onChange={() => setIsPlaylist(false)}
            />{" "}
            Video
          </label>
          <label>
            <input
              type="radio"
              name="type"
              value="playlist"
              checked={isPlaylist}
              onChange={() => setIsPlaylist(true)}
            />{" "}
            Playlist
          </label>
        </div>

        {/* URL Input */}
        <input
          type="text"
          placeholder="Enter YouTube URL..."
          value={videoUrl}
          onChange={(e) => setVideoUrl(e.target.value)}
          className="w-full p-2 mt-2 border border-gray-600 rounded"
        />

        {/* Start Index (Visible only for Playlists) */}
        {isPlaylist && (
          <div className="relative flex items-center mt-2">

            <div className="relative flex items-center mr-2 group">
              Index
              <span className="ml-2 flex items-center justify-center w-5 h-5 text-xs font-bold text-white bg-gray-600 rounded-full cursor-help group-hover:bg-gray-500 transition">
                ?
              </span>

              <div className="absolute left-0 bottom-full mb-2 w-56 p-2 text-xs text-white bg-gray-800 rounded shadow-lg opacity-0 group-hover:opacity-100 transition-opacity duration-300 z-50">
                <span className="block text-center">
                  Enter the video number in the playlist to start from (e.g., 3 for the 3rd video, 78 for the 78th).
                </span>
              </div>
            </div>

            <input
              type="number"
              min="0"
              placeholder="Start Index (optional)"
              value={startIndex}
              onChange={(e) => setStartIndex(Math.max(Number(e.target.value), 1))}
              className="w-full p-2 border border-gray-600 rounded"
            />
          </div>
        )}


        {/* Format Selection */}
        <div className="mt-4">
          <h2 className="text-lg font-semibold">Select Format:</h2>
          {["mp4", "mp3", "wav", "aac", "flac"].map((f) => (
            <label key={f} className="mr-4">
              <input
                type="radio"
                name="format"
                value={f}
                checked={format === f}
                onChange={() => setFormat(f)}
              />{" "}
              {f.toUpperCase()}
            </label>
          ))}
        </div>

        {/* Quality Selection */}
        {format === "mp4" && (
          <div className="mt-4">

            <h2 className="text-lg font-semibold">Select Quality:</h2>

            <p className="text-xs mt-1 mb-1"> If the requested quality is not available, the next best quality will be downloaded.</p>


            {["4k", "1440p", "1080p", "720p", "480p", "360p", "240p", "144p"].map((q) => (
              <label key={q} className="mr-4">
                <input
                  type="radio"
                  name="quality"
                  value={q}
                  checked={quality === q}
                  onChange={() => setQuality(q)}
                />{" "}
                {q.toUpperCase()}
              </label>
            ))}
          </div>
        )}

        {/* Select Folder & Download Button */}
        <div className="flex items-center mt-4">
          <button onClick={selectFolder} className="px-4 py-2 mr-2 bg-gray-700 text-white rounded hover:bg-gray-600 transition">
            Select Folder
          </button>
          <input type="text" placeholder="Download folder..." value={downloadPath} readOnly className="w-full p-2  border border-gray-600 rounded" />
        </div>

        <button onClick={handleDownload} className="w-full py-2 mt-4 bg-emerald-600 text-white rounded hover:bg-emerald-500 transition">
          Download
        </button>


        {/* Divider */}
        <div className="w-full max-w-3xl my-4 border-t border-gray-600"></div>

        {/* Logs Section */}
        <h2 className="text-lg font-semibold">Download Logs</h2>
        <div className="h-32 p-2 mt-2 overflow-y-auto text-sm bg-gray-700 rounded">
          {logs.length === 0 ? (
            <p className="text-gray-400">No logs yet...</p>
          ) : (
            logs.map((log, index) => <p key={index}>{log}</p>)
          )}
        </div>
      </div>

      <LoadingModal isOpen={loading} text={"Downloading..."} finished={finished} onClose={() => setLoading(false)} progress={progress} />
    </main>
  );
}

export default App;