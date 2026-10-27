import { useState } from "react"
import { useQueryClient } from "@tanstack/react-query"
import {
  backupDatabase,
  exportBackup,
  listBackups,
  restoreDatabase,
} from "../../api/setup"
import { useAppInfo } from "../../api/queries"
import {
  Button,
  Card,
  EmptyState,
  ErrorBanner,
  Field,
  Input,
  Loading,
  PageHeader,
} from "../../components"
import { Page } from "../../app/AppShell"
import { formatDateTime } from "../../lib/dates"

export function MaintenancePage() {
  const client = useQueryClient()
  const appInfo = useAppInfo()

  const [backups, setBackups] = useState<string[] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<unknown>(null)
  const [message, setMessage] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [restoring, setRestoring] = useState<string | null>(null)
  const [destinationDirectory, setDestinationDirectory] = useState("")
  const [externalBackupPath, setExternalBackupPath] = useState("")

  const refresh = async () => {
    setLoading(true)
    setError(null)
    try {
      setBackups(await listBackups())
    } catch (failure) {
      setError(failure)
    } finally {
      setLoading(false)
    }
  }

  const takeBackup = async () => {
    setBusy(true)
    setError(null)
    setMessage(null)
    try {
      const path = await backupDatabase()
      setMessage(`Backup written to ${path}`)
      await refresh()
    } catch (failure) {
      setError(failure)
    } finally {
      setBusy(false)
    }
  }

  const restore = async (path: string) => {
    setBusy(true)
    setError(null)
    setMessage(null)
    try {
      await restoreDatabase(path)
      setRestoring(null)
      // The database behind the app has just been replaced, so every cached
      // query is now wrong.
      await client.invalidateQueries()
      await client.refetchQueries()
      setMessage("The database has been restored. Everything has been reloaded.")
    } catch (failure) {
      setError(failure)
    } finally {
      setBusy(false)
    }
  }

  const exportSnapshot = async (path: string) => {
    setBusy(true)
    setError(null)
    setMessage(null)
    try {
      const exported = await exportBackup(path, destinationDirectory.trim())
      setMessage(`Backup exported to ${exported}`)
    } catch (failure) {
      setError(failure)
    } finally {
      setBusy(false)
    }
  }

  return (
    <Page>
      <PageHeader
        title="Maintenance"
        description="Back up the database and restore an earlier snapshot."
        actions={
          <>
            <Button variant="secondary" onClick={refresh} loading={loading}>
              Refresh list
            </Button>
            <Button onClick={takeBackup} loading={busy}>
              Back up now
            </Button>
          </>
        }
      />

      {error ? <ErrorBanner error={error} /> : null}
      {message ? (
        <div className="rounded-md bg-emerald-50 px-4 py-3 text-sm text-emerald-800 ring-1 ring-inset ring-emerald-200">
          {message}
        </div>
      ) : null}

      <Card title="Database">
        <dl className="grid grid-cols-2 gap-4 text-sm sm:grid-cols-4">
          <Item label="Students" value={String(appInfo.data?.studentCount ?? "-")} />
          <Item label="Invoices" value={String(appInfo.data?.invoiceCount ?? "-")} />
          <Item label="Today" value={appInfo.data?.today ?? "-"} />
        </dl>
      </Card>

      <Card title="Backups">
        <div className="mb-4 grid gap-3 sm:grid-cols-[1fr_auto] sm:items-end">
          <Field
            label="Off-device destination folder"
            htmlFor="backup-destination"
            hint="Enter an existing folder on a removable or external drive, such as E:\SchoolBackups."
          >
            <Input
              id="backup-destination"
              value={destinationDirectory}
              onChange={(event) => setDestinationDirectory(event.target.value)}
              placeholder="E:\SchoolBackups"
            />
          </Field>
          <p className="pb-2 text-xs text-slate-500">
            Create a backup first, then export it to this folder.
          </p>
        </div>
        {loading ? (
          <Loading />
        ) : backups === null ? (
          <EmptyState
            title="No list loaded yet"
            description="Refresh the list to see the backups on disk."
            action={<Button onClick={refresh}>Refresh list</Button>}
          />
        ) : backups.length === 0 ? (
          <EmptyState
            title="No backups yet"
            description="Take a backup to protect the data before making changes."
            action={<Button onClick={takeBackup}>Back up now</Button>}
          />
        ) : (
          <ul className="divide-y divide-slate-100">
            {backups.map((path) => (
              <li
                key={path}
                className="flex flex-wrap items-center justify-between gap-3 py-2.5"
              >
                <div className="min-w-0">
                  <p className="truncate font-mono text-xs text-slate-700">
                    {path}
                  </p>
                  <p className="text-xs text-slate-500">
                    {formatDateTime(path)}
                  </p>
                </div>
                <Button
                  size="sm"
                  variant="danger"
                  onClick={() => setRestoring(path)}
                >
                  Restore
                </Button>
                <Button
                  size="sm"
                  variant="secondary"
                  loading={busy}
                  disabled={!destinationDirectory.trim()}
                  onClick={() => exportSnapshot(path)}
                >
                  Export
                </Button>
              </li>
            ))}
          </ul>
        )}
      </Card>

      {restoring ? (
        <Card title="Restore this backup?">
          <p className="text-sm text-slate-600">
            Restoring replaces everything currently in the database with the
            contents of the backup. Anything recorded since that backup will be
            lost.
          </p>
          <p className="mt-2 break-all font-mono text-xs text-slate-500">
            {restoring}
          </p>
          <div className="mt-4 flex gap-2">
            <Button
              variant="danger"
              loading={busy}
              onClick={() => restore(restoring)}
            >
              Restore now
            </Button>
            <Button variant="secondary" onClick={() => setRestoring(null)}>
              Cancel
            </Button>
          </div>
        </Card>
      ) : null}

      <Card title="Restore a backup from another location">
        <div className="grid gap-3 sm:grid-cols-[1fr_auto] sm:items-end">
          <Field
            label="Backup file path"
            htmlFor="external-backup-path"
            hint="Paste the full path to a .db backup on another drive."
          >
            <Input
              id="external-backup-path"
              value={externalBackupPath}
              onChange={(event) => setExternalBackupPath(event.target.value)}
              placeholder="E:\SchoolBackups\school_backup_20261027_090000.db"
            />
          </Field>
          <Button
            variant="danger"
            disabled={!externalBackupPath.trim()}
            onClick={() => setRestoring(externalBackupPath.trim())}
          >
            Select for restore
          </Button>
        </div>
      </Card>
    </Page>
  )
}

function Item({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-xs font-medium uppercase tracking-wide text-slate-500">
        {label}
      </dt>
      <dd className="mt-0.5 text-slate-900">{value}</dd>
    </div>
  )
}
