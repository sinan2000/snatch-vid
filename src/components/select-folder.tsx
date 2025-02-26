import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

interface SelectFolderProps {
  visible: boolean;
  setVisible: (value: boolean) => void;
}

function SelectFolder({ visible, setVisible }: SelectFolderProps) {
  const [path, setPath] = useState<string | null>(null);

  useEffect(() => {
    async function getPath() {
      try {
        const dir = await invoke<string>("read_config");
        setPath(dir);
      } catch (error) {
        console.error(error);
      }
    }
    getPath();
  }, [visible]);

  async function selectFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
    });

    if (typeof selected === "string") setPath(selected);
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
  }

  if (!visible) return null;
  return (
    <div className="fixed inset-0 bg-opacity-50 backdrop-blur-sm z-50 flex justify-center items-center">
      <div className="bg-[#1E1E1E] text-white p-6 rounded-lg max-w-sm w-full shadow-lg">
        <h2 className="text-lg font-semibold text-[#A7F3D0] mb-4 text-center">
          Select a default folder for downloads
        </h2>

        <div className="flex flex-col space-y-3 mb-4">
          <input
            type="text"
            value={path || "No folder selected"}
            readOnly
            className="w-full p-2 text-sm bg-[#2C2C2C] text-gray-300 border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-gray-400"
          />
        </div>

        {/* Buttons Row */}
        <div className="flex gap-3">
          <button
            onClick={selectFolder}
            className="flex-1 bg-[#2A2A2A] text-sm text-[#A7F3D0] py-2 px-3 rounded-lg font-medium hover:bg-gray-700 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-gray-500"
          >
            Choose Folder
          </button>

          <button
            onClick={handleConfirm}
            className="flex-1 bg-[#059669] text-sm text-white py-2 px-3 rounded-lg font-medium hover:bg-[#047857] transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-emerald-400"
          >
            Confirm
          </button>
        </div>
      </div>
    </div>
  );
}