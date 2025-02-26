interface ModalProps {
  isOpen: boolean
  onClose: () => void
  onConfirm: () => void
}

const SelectFolder = ({
  isOpen,
  onClose,
  onConfirm
}: ModalProps) => {
  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-red-500 bg-opacity-50 backdrop-blur-sm flex justify-center items-center">
      <div className="bg-black text-white p-6 rounded-lg max-w-sm w-full">
        <h2 className="text-xl font-bold mb-4">
          Please select the default folder where the videos should be downloaded.
        </h2>
        <button
          onClick={onConfirm}
          className="w-full bg-emerald-500 text-white py-2 px-4 rounded-lg hover:bg-emerald-600 transition-colors"
        >
          Confirm
        </button>
      </div>
    </div>
  )
}

export default SelectFolder;