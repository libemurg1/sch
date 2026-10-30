import { call } from "./client"
import type { AuthStatus } from "../types/schema/AuthStatus"
import type { ChangePasswordInput } from "../types/schema/ChangePasswordInput"
import type { CreateUserInput } from "../types/schema/CreateUserInput"
import type { LoginInput } from "../types/schema/LoginInput"
import type { User } from "../types/schema/User"
import type { UserRole } from "../types/schema/UserRole"

export function getAuthStatus(): Promise<AuthStatus> {
  return call("get_auth_status")
}

export function login(input: LoginInput): Promise<User> {
  return call("login", { input })
}

export function logout(): Promise<void> {
  return call("logout")
}

export function getCurrentUser(): Promise<User | null> {
  return call("get_current_user")
}

export function createUser(input: CreateUserInput): Promise<User> {
  return call("create_user", { input })
}

export function listUsers(): Promise<User[]> {
  return call("list_users")
}

export function changePassword(input: ChangePasswordInput): Promise<void> {
  return call("change_password", { input })
}

export function createAdministratorRecoveryCode(): Promise<string> {
  return call("create_administrator_recovery_code")
}

export function administratorRecoveryConfigured(): Promise<boolean> {
  return call("administrator_recovery_configured")
}

export function recoverAdministrator(
  username: string,
  recoveryCode: string,
  newPassword: string,
): Promise<string> {
  return call("recover_administrator", { username, recoveryCode, newPassword })
}

export function setUserActive(userId: string, isActive: boolean): Promise<void> {
  return call("set_user_active", { userId, isActive })
}

export function updateUser(
  userId: string,
  username: string,
  displayName: string,
  role: UserRole,
): Promise<User> {
  return call("update_user", { userId, username, displayName, role })
}

export function deleteUser(userId: string): Promise<void> {
  return call("delete_user", { userId })
}
