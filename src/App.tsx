import { useState, useEffect, useReducer } from "react";
import "./App.css"
import { invoke } from "@tauri-apps/api/core";
import { Logo } from "./components/logo"
import { Settings, HelpCircle } from "lucide-react"
import SelectFolder from "./components/select-folder";

const initialState = {
  url: "",
  format: "mp4",
  quality: "4k",
};

function reducer(state: any, action: any) {
  return { ...state, [action.name]: action.value };
}

export default function App() {
  const [visible, setVisible] = useState<boolean>(false);
  const [formState, dispatch] = useReducer(reducer, initialState);

  useEffect(() => {
    async function checkConfig() {
      try {
        const exists = await invoke<boolean>("config_exists");
        if (!exists) {
          setVisible(true);
        }
      } catch (error) {
        console.error(error);
      }
    }
    checkConfig();
  }, []);

  async function handleDownload() {
    const validUrlRegex = /^(https?:\/\/)[^\s/$.?#].[^\s]*$/;

    if (!validUrlRegex.test(formState.url)) {
      alert("Invalid URL! Please enter a valid URL starting with http:// or https://");
      return;
    }

    try {
      const type = await invoke<string>("detect_url_type", { url: formState.url });

      console.log("Downloading:", type);
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <main className="flex flex-col items-center justify-center min-h-screen w-full bg-black text-white">
      <div className="w-full p-5">
        <SelectFolder visible={visible} setVisible={setVisible} />

        {/* Top Section */}
        <div className="flex justify-between items-center">
          <div className="flex items-center gap-3">
            <div className="flex flex-col">
              <h1 className="text-2xl font-semibold">SnatchVid</h1>
              <p className="text-sm font-light text-emerald-400 mt-1 tracking-wide">Download any video</p>
            </div>

            <div className="flex items-center gap-2">
              <button className="text-gray-400 hover:text-white transition-colors">
                <Settings size={20} onClick={() => setVisible(true)} />
              </button>
              <button className="text-gray-400 hover:text-white transition-colors">
                <HelpCircle size={20} />
              </button>
            </div>
          </div>

          <a
            href="https://www.snsautomation.tech"
            target="_blank"
            className="group flex items-center gap-2 transition-transform hover:scale-105"
            rel="noreferrer"
          >
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
        <div className="w-full my-4 border-t border-gray-600"></div>

        {/* URL Input */}
        <input
          type="text"
          placeholder="Enter URL..."
          name="url"
          value={formState.url}
          onChange={(e) => dispatch(e.target)}
          className="w-full p-2 mt-2 border border-gray-600 rounded"
        />

        {/* Format Selection */}
        <div className="mt-4">
          <h2 className="text-lg font-semibold">Select Format:</h2>
          <div className="flex gap-4 mt-2">
            {["mp4", "mp3", "wav", "aac", "flac"].map((f) => (
              <label key={f} className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="format"
                  value={f}
                  checked={formState.format === f}
                  onChange={(e) => dispatch(e.target)}
                  className="accent-emerald-500"
                />
                {f.toUpperCase()}
              </label>
            ))}
          </div>
        </div>

        {/* Quality Selection (Only for MP4) */}
        {formState.format === "mp4" && (
          <div className="mt-4">
            <h2 className="text-lg font-semibold">Select Quality:</h2>
            <p className="text-xs mt-1 mb-2">
              If the requested quality is not available, the next best quality will be downloaded.
            </p>

            <div className="flex flex-wrap gap-4">
              {["4k", "1440p", "1080p", "720p", "480p", "360p", "240p", "144p"].map((q) => (
                <label key={q} className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="quality"
                    value={q}
                    checked={formState.quality === q}
                    onChange={(e) => dispatch(e.target)}
                    className="accent-emerald-500"
                  />
                  {q.toUpperCase()}
                </label>
              ))}
            </div>
          </div>
        )}

        {/* Download Button */}
        <button onClick={handleDownload} className="w-full py-2 mt-4 bg-emerald-600 text-white rounded hover:bg-emerald-500 transition">
          Download
        </button>

      </div>
    </main>
  )
}