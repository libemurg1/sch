import { call } from "./client"

export interface LicenseStatus {
  state: "trial" | "active" | "grace" | "readOnly" | "notYetValid"
  activationId: string
  trialDaysRemaining: number
  licenseSchool: string | null
  startsOn: string | null
  expiresOn: string | null
  graceDaysRemaining: number
  message: string
}

export function getLicenseStatus(): Promise<LicenseStatus> {
  return call<LicenseStatus>("get_license_status")
}

export function activateLicense(licenseKey: string): Promise<LicenseStatus> {
  return call<LicenseStatus>("activate_license", { licenseKey })
}
