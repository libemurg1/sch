import { useState } from "react"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import {
  backupDatabase,
  exportBackup,
  listBackups,
  restoreDatabase,
} from "../../api/setup"
import { useAppInfo } from "../../api/queries"
import { useLicenseStatus } from "../../api/queries"
import { activateLicense } from "../../api/license"
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
  const license = useLicenseStatus()
  const [licenseKey, setLicenseKey] = useState("")

  const [backups, setBackups] = useState<string[] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<unknown>(null)
  const [message, setMessage] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [restoring, setRestoring] = useState<string | null>(null)
  const [destinationDirectory, setDestinationDirectory] = useState("")
  const [externalBackupPath, setExternalBackupPath] = useState("")
  const [backupPassphrase, setBackupPassphrase] = useState("")
  const [confirmBackupPassphrase, setConfirmBackupPassphrase] = useState("")
  const [restorePassphrase, setRestorePassphrase] = useState("")
  const activation = useMutation({
    mutationFn: activateLicense,
    onSuccess: async () => {
      setLicenseKey("")
      await client.invalidateQueries({ queryKey: ["license-status"] })
    },
  })
  const portableRestore = restoring?.toLowerCase().endsWith(".sfsbackup") ?? false
  const exportPassphrasesMatch =
    backupPassphrase.length >= 12 &&
    backupPassphrase === confirmBackupPassphrase

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
      setMessage(`Encrypted local backup written to ${path}`)
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
      await restoreDatabase(path, portableRestore ? restorePassphrase : undefined)
      setRestoring(null)
      setRestorePassphrase("")
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
      const exported = await exportBackup(
        path,
        destinationDirectory.trim(),
        backupPassphrase,
      )
      setBackupPassphrase("")
      setConfirmBackupPassphrase("")
      setMessage(`Encrypted portable backup exported to ${exported}`)
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

      <Card
        title="License"
        description="This offline license is tied to this installation. Provide the activation ID to your software provider to receive a signed key."
      >
        {license.error ? <ErrorBanner error={license.error} onRetry={() => license.refetch()} /> : null}
        {license.data ? (
          <div className="space-y-4">
            <p className="text-sm text-slate-700">
              {license.data.message}
              {license.data.licenseSchool ? ` Licensed to ${license.data.licenseSchool}.` : ""}
              {license.data.expiresOn ? ` Expiry date: ${license.data.expiresOn}.` : ""}
            </p>
            <Field
              label="Device activation ID"
              htmlFor="license-activation-id"
              hint="Send this ID to the license issuer. Do not send your database or private signing key."
            >
              <Input
                id="license-activation-id"
                value={license.data.activationId}
                readOnly
                onFocus={(event) => event.currentTarget.select()}
              />
            </Field>
            <Field
              label="License key"
              htmlFor="license-key"
              hint="Paste the signed key supplied for this activation ID."
            >
              <Input
                id="license-key"
                value={licenseKey}
                onChange={(event) => setLicenseKey(event.target.value)}
                autoComplete="off"
                spellCheck={false}
                placeholder="sfs1.…"
              />
            </Field>
            {activation.error ? <ErrorBanner error={activation.error} /> : null}
            {activation.isSuccess ? (
              <p className="text-sm text-emerald-700">License activated successfully.</p>
            ) : null}
            <Button
              onClick={() => activation.mutate(licenseKey.trim())}
              loading={activation.isPending}
              disabled={!licenseKey.trim()}
            >
              Activate license
            </Button>
          </div>
        ) : license.isLoading ? (
          <Loading />
        ) : null}
      </Card>

      <Card title="Database">
        <dl className="grid grid-cols-2 gap-4 text-sm sm:grid-cols-4">
          <Item label="Students" value={String(appInfo.data?.studentCount ?? "-")} />
          <Item label="Invoices" value={String(appInfo.data?.invoiceCount ?? "-")} />
          <Item label="Today" value={appInfo.data?.today ?? "-"} />
        </dl>
      </Card>

      <Card title="Backups">
        <p className="mb-4 text-sm text-slate-600">
          Local backups are encrypted for this Windows account. Exports are
          encrypted with a passphrase and can be restored on another Windows
          account or computer. Keep the passphrase separate from the backup;
          it cannot be recovered if lost.
        </p>
        <div className="mb-4 grid gap-3 sm:grid-cols-2">
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
          <Field
            label="Portable backup passphrase"
            htmlFor="backup-passphrase"
            hint="Use at least 12 characters. Store it separately from the backup."
            required
          >
            <Input
              id="backup-passphrase"
              type="password"
              autoComplete="new-password"
              value={backupPassphrase}
              onChange={(event) => setBackupPassphrase(event.target.value)}
              placeholder="At least 12 characters"
            />
          </Field>
          <Field
            label="Confirm passphrase"
            htmlFor="confirm-backup-passphrase"
            error={
              confirmBackupPassphrase && !exportPassphrasesMatch
                ? "Passphrases must match and be at least 12 characters."
                : undefined
            }
            required
          >
            <Input
              id="confirm-backup-passphrase"
              type="password"
              autoComplete="new-password"
              value={confirmBackupPassphrase}
              onChange={(event) =>
                setConfirmBackupPassphrase(event.target.value)
              }
              placeholder="Re-enter the passphrase"
            />
          </Field>
          <p className="self-end pb-2 text-xs text-slate-500">
            Create a backup first, then export its encrypted copy.
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
                  disabled={
                    !destinationDirectory.trim() || !exportPassphrasesMatch
                  }
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
          {portableRestore ? (
            <Field
              className="mt-4"
              label="Backup passphrase"
              htmlFor="restore-passphrase"
              hint="Use the passphrase that was set when this backup was exported."
              required
            >
              <Input
                id="restore-passphrase"
                type="password"
                autoComplete="current-password"
                value={restorePassphrase}
                onChange={(event) => setRestorePassphrase(event.target.value)}
                placeholder="Portable backup passphrase"
              />
            </Field>
          ) : null}
          <p className="mt-2 break-all font-mono text-xs text-slate-500">
            {restoring}
          </p>
          <div className="mt-4 flex gap-2">
            <Button
              variant="danger"
              loading={busy}
              disabled={portableRestore && restorePassphrase.length < 12}
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
            hint="Choose an encrypted .sfsbackup export for portability, or a .db backup from this Windows account."
          >
            <Input
              id="external-backup-path"
              value={externalBackupPath}
              onChange={(event) => setExternalBackupPath(event.target.value)}
              placeholder="E:\SchoolBackups\school_backup_20261027_090000.sfsbackup"
            />
          </Field>
          <Button
            variant="danger"
            disabled={!externalBackupPath.trim()}
            onClick={() => {
              setRestorePassphrase("")
              setRestoring(externalBackupPath.trim())
            }}
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
