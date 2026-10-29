import type { ReactNode } from "react"
import { useSchoolSettings } from "../../app/session"
import { DeveloperNote } from "../../components"

/** The centred, branded frame shared by the sign-in and first-run screens. */
export function AuthLayout({
  subtitle,
  children,
}: {
  subtitle: string
  children: ReactNode
}) {
  const { settings } = useSchoolSettings()
  const schoolName = settings?.schoolName ?? "SFS"

  return (
    <div className="flex min-h-screen items-center justify-center bg-slate-100 px-4 py-10">
      <div className="w-full max-w-md">
        <div className="mb-6 flex flex-col items-center text-center">
          {settings?.logoData ? (
            <img
              src={settings.logoData}
              alt={`${schoolName} logo`}
              className="mb-3 h-16 max-w-[200px] object-contain"
            />
          ) : (
            <div className="mb-3 flex h-12 w-12 items-center justify-center rounded-lg bg-indigo-600 text-sm font-bold text-white">
              <span className="whitespace-nowrap">SFS</span>
            </div>
          )}
          <h1 className="text-xl font-semibold tracking-tight text-slate-900">
            {schoolName}
          </h1>
          {settings?.address ? (
            <p className="mt-1 text-xs text-slate-500">{settings.address}</p>
          ) : null}
          <p className="mt-2 text-sm text-slate-600">{subtitle}</p>
        </div>

        <div className="rounded-lg bg-white px-6 py-6 shadow-sm ring-1 ring-slate-200">
          {children}
        </div>

        <p className="mt-6 text-center text-xs text-slate-400">{schoolName}</p>
        <div className="mt-4 flex justify-center text-center">
          <DeveloperNote />
        </div>
      </div>
    </div>
  )
}
