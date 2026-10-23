/**
 * The backend rejects a failed command with the serialised `AppError`, so
 * every rejection arrives as `{ code, message }`. This module turns that into
 * a real `Error` subclass the UI can branch on.
 */

/** Every error code the backend can produce. */
export type ApiErrorCode =
  | "VALIDATION"
  | "NOT_FOUND"
  | "CONFLICT"
  | "NO_FEE_STRUCTURE"
  | "DB_ERROR"
  | "IO_ERROR"
  | "NOT_IMPLEMENTED"
  | "UNKNOWN"

/** A failed Tauri command. */
export class ApiError extends Error {
  readonly code: ApiErrorCode

  constructor(code: ApiErrorCode, message: string) {
    super(message)
    this.name = "ApiError"
    this.code = code
  }

  /** True when the user needs to change what they typed. */
  get isValidation(): boolean {
    return this.code === "VALIDATION"
  }

  /** True when the request clashed with the current state of the data. */
  get isConflict(): boolean {
    return this.code === "CONFLICT" || this.code === "NO_FEE_STRUCTURE"
  }

  /** True when the thing the user asked for does not exist. */
  get isNotFound(): boolean {
    return this.code === "NOT_FOUND"
  }
}

const KNOWN_CODES: ApiErrorCode[] = [
  "VALIDATION",
  "NOT_FOUND",
  "CONFLICT",
  "NO_FEE_STRUCTURE",
  "DB_ERROR",
  "IO_ERROR",
  "NOT_IMPLEMENTED",
]

function isErrorCode(value: string): value is ApiErrorCode {
  return (KNOWN_CODES as string[]).includes(value)
}

/**
 * Converts anything thrown by `invoke` into an `ApiError`.
 *
 * A rejection that is not an `AppError` still has to render as something
 * useful, so it becomes `UNKNOWN` with the original text as the message.
 */
export function toApiError(error: unknown): ApiError {
  if (error instanceof ApiError) {
    return error
  }

  if (typeof error === "string") {
    return new ApiError("UNKNOWN", error)
  }

  if (typeof error === "object" && error !== null) {
    const candidate = error as { code?: unknown; message?: unknown }
    const code = typeof candidate.code === "string" ? candidate.code : ""
    const message =
      typeof candidate.message === "string" && candidate.message.length > 0
        ? candidate.message
        : "The command failed for an unknown reason."

    if (isErrorCode(code)) {
      return new ApiError(code, message)
    }
    if (code === "") {
      return new ApiError("UNKNOWN", message)
    }
    return new ApiError("UNKNOWN", `${code}: ${message}`)
  }

  return new ApiError("UNKNOWN", "The command failed for an unknown reason.")
}

/** A short, human-readable summary of an error for a banner or toast. */
export function describeError(error: unknown): string {
  const apiError = toApiError(error)
  switch (apiError.code) {
    case "VALIDATION":
      return apiError.message
    case "NOT_FOUND":
      return apiError.message
    case "CONFLICT":
      return apiError.message
    case "NO_FEE_STRUCTURE":
      return "No fee structure has been set for this class and term yet."
    case "NOT_IMPLEMENTED":
      return apiError.message
    default:
      return apiError.message
  }
}
