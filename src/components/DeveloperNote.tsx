import { invoke } from "@tauri-apps/api/core"

export function DeveloperNote({ compact = false }: { compact?: boolean }) {
  const openLinkedIn = () => {
    void invoke("open_developer_profile").catch(() => {
      // Nothing useful to show if the browser cannot be opened.
    })
  }

  return (
    <div className={compact ? "px-3" : ""}>
      <p className="whitespace-nowrap text-xs font-medium text-slate-700">
        Developer: Liberty Magumise
      </p>
      <p className="mt-0.5 flex justify-center gap-x-2 whitespace-nowrap text-xs text-slate-500">
        <a
          href="tel:+263774620179"
          className="font-semibold text-indigo-700 hover:underline"
          aria-label="Call developer on +263 774 620 179"
        >
          +263 774 620 179
        </a>
        <span aria-hidden="true">·</span>
        <button
          type="button"
          onClick={openLinkedIn}
          className="text-indigo-600 hover:underline"
        >
          LinkedIn
        </button>
      </p>
    </div>
  )
}
