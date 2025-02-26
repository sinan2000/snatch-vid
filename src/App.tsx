import "./App.css"
import { Logo } from "./components/logo"

function App() {
  return (
    <main className="flex flex-col items-center justify-center min-h-screen w-full bg-black text-white">
      <div className="w-full p-5">
        {/* Top Section */}
        <div className="flex justify-between items-center">
          <div className="flex flex-col">
            <h1 className="text-2xl font-semibold">SnatchVid</h1>
            <p className="text-sm font-light text-emerald-400 mt-1 tracking-wide">
              Download any video
            </p>
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
      </div>
    </main>
  )
}

export default App

