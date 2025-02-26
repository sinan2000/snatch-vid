import { Loader2, CheckCircle } from 'lucide-react'

interface LoadingModalProps {
  text: string
  phase: number
  onClose: (value: number) => void
  progress: number
}

export function LoadingModal({ text, phase, onClose, progress }: LoadingModalProps) {
  const finished = phase === 3 || phase === 4
  if (phase === 0) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" />
      <div className="relative bg-[#1E1E1E] p-6 rounded-lg shadow-lg flex flex-col items-center">
        {!finished ? (
          <Loader2 className="w-12 h-12 text-emerald-500 animate-spin" />
        ) : (
          <CheckCircle className="w-12 h-12 text-emerald-500 animate-bounce" />
        )}

        <p className="text-white text-lg mt-4 mb-2 dots">
          {text}
        </p>

        {phase === 2 && (
          <div className="w-full max-w-3xl my-4">
            <progress value={progress} max="100" className="w-full" />
            <span className="ml-2">{progress}%</span>
          </div>
        )}

        {finished && (
          <div className="mt-4 flex items-center flex-col">

            <button
              className="px-4 py-2 bg-emerald-600 text-white rounded hover:bg-emerald-500 transition"
              onClick={() => onClose(0)}
            >
              Close
            </button>
          </div>
        )}
      </div >
    </div >
  )
}
