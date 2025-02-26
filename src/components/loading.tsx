import { Loader2, CheckCircle } from 'lucide-react'

interface LoadingModalProps {
  isOpen: boolean
  text: string
  finished: boolean
  onClose: () => void
  progress: number
}

export function LoadingModal({ isOpen, text, finished, onClose, progress }: LoadingModalProps) {
  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" />
      <div className="relative bg-black p-6 rounded-lg shadow-lg flex flex-col items-center">
        {!finished ? (
          <>
            <Loader2 className="w-12 h-12 text-emerald-500 animate-spin" />
            <div className="w-full max-w-3xl my-4">
              <progress value={progress} max="100" className="w-full" />
              <span className="ml-2">{progress}%</span>
            </div>
          </>
        ) : (
          <CheckCircle className="w-12 h-12 text-emerald-500 animate-bounce" />
        )}

        {!finished ? (
          <p className="mt-4 text-white text-lg">{text}</p>
        ) : (
          <div className="mt-4 flex items-center flex-col">
            <p className="text-white text-lg mb-2">
              Download finished successfully
            </p>
            <button
              className="px-4 py-2 bg-emerald-600 text-white rounded hover:bg-emerald-500 transition"
              onClick={onClose}
            >
              Close
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
