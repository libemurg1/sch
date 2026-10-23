import { call } from "./client"
import type { AuthStatus } from "../types/schema/AuthStatus"
import type { ChangePasswordInput } from "../types/schema/ChangePasswordInput"
import type { CreateUserInput } from "../types/schema/CreateUserInput"
import type { LoginInput } from "../types/schema/LoginInput"
import type { User } from "../types/schema/User"

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

export function setUserActive(userId: string, isActive: boolean): Promise<void> {
  return call("set_user_active", { userId, isActive })
}

export function updateUser(
  userId: string,
  username: string,
  displayName: string,
): Promise<User> {
  return call("update_user", { userId, username, displayName })
}

export function deleteUser(userId: string): Promise<void> {
  return call("delete_user", { userId })
}
