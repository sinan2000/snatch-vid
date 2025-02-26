import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

function SelectFolder() {
  const [visible, setVisible] = useState<boolean>(true);
  const [path, setPath] = useState<string | null>(null);

  useEffect(() => {
    async function checkConfig() {
      try {
        const exists = await invoke<boolean>("config_exists");
        if (exists) {
          setVisible(false);
        }
      } catch (error) {
        console.error(error);
      }
    }
    checkConfig();
  }, []);

  async function selectFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
    });

    if (typeof selected === "string")
      setPath(selected);
  }

  async function handleConfirm() {
    if (path) {
      try {
        await invoke("create_config", { dir: path });
        setVisible(false);
      } catch (error) {
        console.error(error);
      }
    }
  };

  if (!visible) return null

  return (
    <div className="fixed inset-0 bg-opacity-50 backdrop-blur-sm flex justify-center items-center">
      <div className="bg-black text-white p-8 rounded-xl max-w-md w-full shadow-lg">
        <h2 className="text-2xl font-bold mb-6">
          Please select the default folder where the videos should be downloaded.
        </h2>

        <div className="flex flex-col space-y-4 mb-6">
          <button
            onClick={selectFolder}
            className="w-full bg-gray-700 text-white py-3 px-4 rounded-lg font-semibold hover:bg-gray-600 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-gray-400"
          >
            Select Folder
          </button>
          <input
            type="text"
            value={path || "Download folder..."}
            readOnly
            className="w-full p-3 bg-gray-800 text-gray-300 border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-gray-400"
          />
        </div>

        <button
          onClick={handleConfirm}
          className="w-full bg-emerald-500 text-white py-3 px-4 rounded-lg font-semibold hover:bg-emerald-600 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-emerald-400"
        >
          Confirm
        </button>
      </div>
    </div>
  )
}

export default SelectFolder;