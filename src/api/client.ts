/**
 * The single door through which the React app reaches the Rust backend.
 *
 * Nothing in `src/` may talk to the database directly (see README "Core
 * Principles"); every call goes through a Tauri command.
 */
import { invoke } from "@tauri-apps/api/core"
import { toApiError } from "./errors"

/**
 * Calls a Tauri command and normalises any failure into an `ApiError`.
 *
 * Tauri camel-cases command arguments by default, so a Rust parameter named
 * `student_id` is called as `{ studentId }` from here.
 */
export async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    throw toApiError(error)
  }
}
