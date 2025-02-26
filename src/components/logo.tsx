import { cn } from "./util/cn";

export function Logo({ className }: { className?: string }) {
  return (
    <div className={cn("relative group", className)}>
      <div className="absolute -inset-[2px] bg-gradient-to-r from-emerald-400 via-emerald-500 to-cyan-400 rounded-lg blur-sm opacity-75 group-hover:opacity-100 transition duration-500"></div>
      <div className="relative flex items-center bg-black rounded-lg p-1">
        <svg
          width="32"
          height="32"
          viewBox="0 0 32 32"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          className="text-emerald-400"
        >
          {/* Main Thunder Bolt */}
          <path
            d="M18 4L8 18H16L14 28L24 14H16L18 4Z"
            className="stroke-current"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <animate
              attributeName="opacity"
              values="1;0.7;1"
              dur="0.5s"
              repeatCount="indefinite"
            />
          </path>

          {/* Energy Particles */}
          <circle
            cx="20"
            cy="12"
            r="1"
            className="fill-cyan-400 animate-ping"
            style={{ animationDuration: '1s' }}
          />
          <circle
            cx="12"
            cy="20"
            r="1"
            className="fill-emerald-400 animate-ping"
            style={{ animationDuration: '1.5s' }}
          />

          {/* Electric Field Lines */}
          <path
            d="M6 16C10 14 14 14 16 16"
            className="stroke-current"
            strokeWidth="1"
            strokeLinecap="round"
            opacity="0.5"
          >
            <animate
              attributeName="d"
              values="M6 16C10 14 14 14 16 16;M6 16C10 18 14 18 16 16;M6 16C10 14 14 14 16 16"
              dur="2s"
              repeatCount="indefinite"
            />
          </path>
          <path
            d="M16 16C18 18 22 18 26 16"
            className="stroke-current"
            strokeWidth="1"
            strokeLinecap="round"
            opacity="0.5"
          >
            <animate
              attributeName="d"
              values="M16 16C18 18 22 18 26 16;M16 16C18 14 22 14 26 16;M16 16C18 18 22 18 26 16"
              dur="2s"
              repeatCount="indefinite"
            />
          </path>

          {/* Power Sparks */}
          <path
            d="M8 8L10 10M24 8L22 10M16 2L16 4"
            className="stroke-current"
            strokeWidth="1.5"
            strokeLinecap="round"
          >
            <animate
              attributeName="opacity"
              values="0.2;1;0.2"
              dur="1.5s"
              repeatCount="indefinite"
            />
          </path>
        </svg>
      </div>
    </div>
  );
}