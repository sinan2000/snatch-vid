import { Loader2, CheckCircle, XCircle } from 'lucide-react'

interface LoadingModalProps {
  text: string
  phase: number
  onClose: () => void
  progress: number
}

export default function LoadingModal({ text, phase, onClose, progress }: LoadingModalProps) {
  const success = phase === 3;
  const error = phase === 4;
  const finished = success || error;
  
  if (phase === 0) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" />
      <div className="relative bg-[#1E1E1E] p-6 rounded-lg shadow-lg flex flex-col items-center">
        {!finished ? (
          <Loader2 className="w-12 h-12 text-emerald-500 animate-spin" />
        ) : success ? (
          <CheckCircle className="w-12 h-12 text-emerald-500 animate-bounce" />
        ) : (
          <XCircle className="w-12 h-12 text-red-500 animate-bounce" />
        )}

        <p className={`text-white text-lg mt-4 mb-2 ${!finished ? 'dots' : ''}`}>
          {text}
        </p>

        {phase === 2 && (
          <div className="w-full max-w-3xl my-4 relative">
            {/* Background Bar */}
            <div className="w-full h-6 bg-gray-700 rounded-full overflow-hidden">
              {/* Progress Fill */}
              <div
                className="h-full rounded-full transition-all duration-300"
                style={{
                  width: `${progress}%`,
                  background: "linear-gradient(to right, #059669, #34D399)",
                }}
              />
            </div>

            {/* Percentage Indicator Inside */}
            <span
              className="absolute inset-0 flex justify-center items-center text-sm font-semibold transition-colors"
              style={{ color: "#FFFFFF" }}
            >
              {progress}%
            </span>
          </div>
        )}


        {finished && (
          <div className="mt-4 flex items-center flex-col">

            <button
              className="px-4 py-2 bg-emerald-600 text-white rounded hover:bg-emerald-500 transition"
              onClick={() => onClose()}
            >
              Close
            </button>
          </div>
        )}
      </div >
    </div >
  )
}
