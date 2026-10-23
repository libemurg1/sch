import { call } from "./client"
import type { SchoolSettings } from "../types/schema/SchoolSettings"
import type { UpdateSchoolSettingsInput } from "../types/schema/UpdateSchoolSettingsInput"

/**
 * Public command: the login screen shows the school name and logo before the
 * user has authenticated.
 */
export function getSchoolSettings(): Promise<SchoolSettings> {
  return call("get_school_settings")
}

export function updateSchoolSettings(
  input: UpdateSchoolSettingsInput,
): Promise<SchoolSettings> {
  return call("update_school_settings", { input })
}

/** `logoData` is a `data:image/...;base64,...` URI, or `null` to clear it. */
export function setLogo(logoData: string | null): Promise<SchoolSettings> {
  return call("set_logo", { logoData })
}
