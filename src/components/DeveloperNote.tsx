import { invoke } from "@tauri-apps/api/core"

/**
 * Support details for the developer. Kept on every signed-in screen so a
 * school can always find out who to contact about the system.
 */
export function DeveloperNote({ compact = false }: { compact?: boolean }) {
  const openLinkedIn = () => {
    void invoke("open_developer_profile").catch(() => {
      // Nothing useful to show if the browser cannot be opened.
    })
  }

  return (
    <div className={compact ? "px-3" : ""}>
      <p className="text-[11px] font-semibold uppercase tracking-wider text-slate-400">
        Developer
      </p>
      <p className="mt-1 text-xs font-medium text-slate-700">
        Liberty Magumise
      </p>
      <ul className="mt-1 space-y-0.5 text-xs text-slate-500">
        <li>
          <a
            href="tel:+263774620179"
            className="hover:text-indigo-600 hover:underline"
          >
            +263 774 620 179
          </a>
        </li>
        <li>
          <a
            href="mailto:libemurg@gmail.com"
            className="hover:text-indigo-600 hover:underline"
          >
            libemurg@gmail.com
          </a>
        </li>
        <li>
          <button
            type="button"
            onClick={openLinkedIn}
            className="text-left text-indigo-600 hover:underline"
          >
            linkedin.com/in/libemurg
          </button>
        </li>
      </ul>
    </div>
  )
}
