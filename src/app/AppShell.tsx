import { useState, type ReactNode } from "react"
import { NavLink, Outlet } from "react-router-dom"
import { useActiveTerm, useAppInfo } from "../api/queries"
import { DeveloperNote } from "../components"
import { useSchoolSettings, useSession } from "./session"
import { cx } from "../lib/cx"
import { formatDate, formatTermLabel } from "../lib/dates"

interface NavItem {
  to: string
  label: string
  end?: boolean
}

interface NavGroup {
  group: string
  items: NavItem[]
}

const NAV: NavGroup[] = [
  { group: "Overview", items: [{ to: "/", label: "Dashboard", end: true }] },
  {
    group: "Records",
    items: [
      { to: "/students", label: "Students" },
      { to: "/classes", label: "Classes" },
      { to: "/terms", label: "Terms" },
      { to: "/enrollment", label: "Enrollment" },
      { to: "/fee-structures", label: "Fee Structures" },
      { to: "/discounts", label: "Discounts" },
    ],
  },
  {
    group: "Money",
    items: [
      { to: "/billing", label: "Billing" },
      { to: "/payments", label: "Payments" },
      { to: "/statements", label: "Statements" },
    ],
  },
  {
    group: "System",
    items: [
      { to: "/settings", label: "Settings" },
      { to: "/users", label: "Users" },
      { to: "/maintenance", label: "Maintenance" },
    ],
  },
]

/** The frame every screen sits in: navigation, header and the page outlet. */
export function AppShell() {
  const [menuOpen, setMenuOpen] = useState(false)
  const appInfo = useAppInfo()
  const activeTerm = useActiveTerm()
  const { settings } = useSchoolSettings()
  const { user, logout } = useSession()

  const schoolName = settings?.schoolName ?? "School Fees"
  const logo = settings?.logoData

  const brand = (
    <>
      {logo ? (
        <img
          src={logo}
          alt=""
          className="h-7 w-7 shrink-0 rounded object-contain"
        />
      ) : (
        <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded bg-indigo-600 text-[10px] font-bold text-white">
          SFS
        </div>
      )}
      <span className="truncate text-sm font-semibold text-slate-900">
        {schoolName}
      </span>
    </>
  )

  const navigation = (
    <nav className="flex-1 space-y-6 overflow-y-auto px-3 py-4">
      {NAV.map((section) => (
        <div key={section.group}>
          <p className="px-3 pb-2 text-[11px] font-semibold uppercase tracking-wider text-slate-400">
            {section.group}
          </p>
          <ul className="space-y-0.5">
            {section.items.map((item) => (
              <li key={item.to}>
                <NavLink
                  to={item.to}
                  end={item.end}
                  onClick={() => setMenuOpen(false)}
                  className={({ isActive }) =>
                    cx(
                      "block rounded-md px-3 py-2 text-sm font-medium transition-colors",
                      isActive
                        ? "bg-indigo-50 text-indigo-700"
                        : "text-slate-600 hover:bg-slate-100 hover:text-slate-900",
                    )
                  }
                >
                  {item.label}
                </NavLink>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </nav>
  )

  return (
    <div className="min-h-screen bg-slate-50">
      <aside className="no-print fixed inset-y-0 left-0 z-30 hidden w-60 flex-col border-r border-slate-200 bg-white lg:flex">
        <div className="flex h-14 items-center gap-2 border-b border-slate-200 px-5">
          {brand}
        </div>
        {navigation}
        <div className="border-t border-slate-200 px-3 py-3">
          <p className="truncate px-3 text-xs text-slate-500">
            Signed in as{" "}
            <span className="font-medium text-slate-700">
              {user?.displayName ?? "-"}
            </span>
          </p>
          <div className="mt-3 border-t border-slate-100 px-3 pt-3">
            <DeveloperNote />
          </div>
        </div>
      </aside>

      {menuOpen ? (
        <div className="no-print fixed inset-0 z-40 lg:hidden">
          <div
            className="absolute inset-0 bg-slate-900/40"
            onClick={() => setMenuOpen(false)}
          />
          <div className="absolute inset-y-0 left-0 flex w-64 flex-col bg-white shadow-xl">
            <div className="flex h-14 items-center gap-2 border-b border-slate-200 px-5">
              {brand}
            </div>
            {navigation}
            <div className="border-t border-slate-200 px-3 py-3">
              <p className="truncate px-3 text-xs text-slate-500">
                Signed in as{" "}
                <span className="font-medium text-slate-700">
                  {user?.displayName ?? "-"}
                </span>
              </p>
            </div>
          </div>
        </div>
      ) : null}

      <div className="lg:pl-60">
        <header className="no-print sticky top-0 z-20 flex h-14 items-center gap-3 border-b border-slate-200 bg-white/95 px-4 backdrop-blur sm:px-6">
          <button
            type="button"
            onClick={() => setMenuOpen(true)}
            className="rounded-md p-1.5 text-slate-500 hover:bg-slate-100 lg:hidden"
            aria-label="Open navigation"
          >
            <svg
              className="h-5 w-5"
              viewBox="0 0 20 20"
              fill="currentColor"
              aria-hidden="true"
            >
              <path d="M3 5.5A1.5 1.5 0 014.5 4h11a1.5 1.5 0 011.5 1.5v1A1.5 1.5 0 0115.5 8h-11A1.5 1.5 0 013 6.5v-1zm0 5A1.5 1.5 0 014.5 9h11a1.5 1.5 0 011.5 1.5v1A1.5 1.5 0 0115.5 13h-11A1.5 1.5 0 013 11.5v-1zm0 5A1.5 1.5 0 014.5 14h11a1.5 1.5 0 011.5 1.5v1A1.5 1.5 0 0115.5 18h-11A1.5 1.5 0 013 16.5v-1z" />
            </svg>
          </button>

          <div className="flex-1" />

          <div className="flex items-center gap-4 text-xs text-slate-500">
            <span className="hidden sm:inline">
              {activeTerm.data
                ? `Active: ${formatTermLabel(activeTerm.data)}`
                : "No active term"}
            </span>
            <span className="hidden sm:inline">
              {appInfo.data ? formatDate(appInfo.data.today) : "-"}
            </span>
            <div className="flex items-center gap-2 border-l border-slate-200 pl-4">
              <span className="hidden max-w-[12rem] truncate font-medium text-slate-700 md:inline">
                {user?.displayName ?? ""}
              </span>
              <button
                type="button"
                onClick={() => logout.mutate()}
                disabled={logout.isPending}
                className="rounded-md px-2 py-1 font-medium text-slate-600 ring-1 ring-inset ring-slate-300 hover:bg-slate-100 disabled:text-slate-400"
              >
                Sign out
              </button>
            </div>
          </div>
        </header>

        <main className="px-4 py-6 sm:px-6 lg:px-8">
          <Outlet />
        </main>
      </div>
    </div>
  )
}

/** A thin page wrapper that keeps the spacing consistent across screens. */
export function Page({ children }: { children: ReactNode }) {
  return <div className="mx-auto max-w-7xl space-y-6">{children}</div>
}
