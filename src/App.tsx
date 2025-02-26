import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  async function getDownloadPath() {
    const path = await invoke<string>("get_download_directory");
    console.log("Download Path:", path);
  }

  async function getBinaryPaths() {
    const [ytDlp, ffmpeg] = await invoke<[string, string]>("get_binaries");
    console.log("yt-dlp:", ytDlp);
    console.log("ffmpeg:", ffmpeg);
  }  

  useEffect(() => {
    getDownloadPath();
    getBinaryPaths();
  }, []);

  return <p>Hello world!</p>
}

export default App;