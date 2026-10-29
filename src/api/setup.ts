import { call } from "./client"
import type { AppInfo } from "../types/schema/AppInfo"
import type { DatabaseHealth } from "../types/schema/DatabaseHealth"

/** Counts and the server's idea of today. */
export function getAppInfo(): Promise<AppInfo> {
  return call<AppInfo>("get_app_info")
}

export function getDatabaseHealth(): Promise<DatabaseHealth> {
  return call<DatabaseHealth>("get_database_health")
}

/** Takes an online snapshot of the database and returns its path. */
export function backupDatabase(): Promise<string> {
  return call<string>("backup_database")
}

/** The paths of every backup taken so far. */
export function listBackups(): Promise<string[]> {
  return call<string[]>("list_backups")
}

/** Copies one application backup into an existing user-selected folder. */
export function exportBackup(
  backupPath: string,
  destinationDirectory: string,
  passphrase: string,
): Promise<string> {
  return call<string>("export_backup", {
    backupPath,
    destinationDirectory,
    passphrase,
  })
}

/**
 * Restores the database from a backup file.
 *
 * This replaces the live database, so the caller has to drop every cached
 * query and reload afterwards.
 */
export function restoreDatabase(
  backupPath: string,
  passphrase?: string,
): Promise<void> {
  return call<void>("restore_database", { backupPath, passphrase })
}
