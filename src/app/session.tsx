import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseMutationResult,
} from "@tanstack/react-query"
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  type ReactNode,
} from "react"
import {
  createUser,
  getAuthStatus,
  login as loginRequest,
  logout as logoutRequest,
} from "../api/auth"
import { getSchoolSettings, updateSchoolSettings } from "../api/settings"
import { configureCurrency } from "../lib/money"
import type { AuthStatus } from "../types/schema/AuthStatus"
import type { CreateUserInput } from "../types/schema/CreateUserInput"
import type { LoginInput } from "../types/schema/LoginInput"
import type { SchoolSettings } from "../types/schema/SchoolSettings"
import type { User } from "../types/schema/User"

/* ------------------------------------------------------------------- auth */

/** The bootstrap form also collects the school's name for the printed header. */
export type BootstrapInput = CreateUserInput & { schoolName?: string }

interface SessionContextValue {
  status: AuthStatus | undefined
  user: User | null
  hasUsers: boolean
  loading: boolean
  error: unknown
  login: UseMutationResult<User, unknown, LoginInput>
  logout: UseMutationResult<void, unknown, void>
  bootstrap: UseMutationResult<User, unknown, BootstrapInput>
  refresh: () => Promise<void>
}

const SessionContext = createContext<SessionContextValue | null>(null)

export function SessionProvider({ children }: { children: ReactNode }) {
  const client = useQueryClient()

  const statusQuery = useQuery({
    queryKey: ["auth"],
    queryFn: getAuthStatus,
    staleTime: Infinity,
    retry: false,
  })

  const login = useMutation({
    mutationFn: loginRequest,
    onSuccess: async () => {
      // Everything cached before sign-in was fetched unauthenticated (or not
      // at all), so start from a clean slate with the new identity.
      await client.invalidateQueries()
      await client.refetchQueries()
    },
  })

  const logout = useMutation({
    mutationFn: logoutRequest,
    onSuccess: () => {
      client.clear()
    },
  })

  const bootstrap = useMutation({
    mutationFn: async (input: BootstrapInput) => {
      const user = await createUser(input)
      await loginRequest({
        username: input.username,
        password: input.password,
        rememberMe: false,
      })

      // Now authenticated, so the school name can be stored for the letterhead.
      const schoolName = input.schoolName?.trim()
      if (schoolName) {
        const current = await getSchoolSettings()
        await updateSchoolSettings({
          schoolName,
          address: current.address,
          phone: current.phone,
          email: current.email,
          currencyCode: current.currencyCode,
          currencySymbol: current.currencySymbol,
          bankName: current.bankName,
          bankAccountName: current.bankAccountName,
          bankAccountNumber: current.bankAccountNumber,
          bankBranch: current.bankBranch,
          bankSwift: current.bankSwift,
          receiptFooter: current.receiptFooter,
          invoiceFooter: current.invoiceFooter,
        })
      }

      return user
    },
    onSuccess: async () => {
      await client.invalidateQueries()
      await client.refetchQueries()
    },
  })

  const refresh = useCallback(async () => {
    await statusQuery.refetch()
  }, [statusQuery])

  const value = useMemo<SessionContextValue>(
    () => ({
      status: statusQuery.data,
      user: statusQuery.data?.user ?? null,
      hasUsers: statusQuery.data?.hasUsers ?? true,
      loading: statusQuery.isLoading,
      error: statusQuery.error,
      login,
      logout,
      bootstrap,
      refresh,
    }),
    [statusQuery.data, statusQuery.isLoading, statusQuery.error, login, logout, bootstrap, refresh],
  )

  return (
    <SessionContext.Provider value={value}>{children}</SessionContext.Provider>
  )
}

export function useSession(): SessionContextValue {
  const context = useContext(SessionContext)
  if (!context) {
    throw new Error("useSession must be used inside <SessionProvider>")
  }
  return context
}

/* --------------------------------------------------------------- settings */

interface SettingsContextValue {
  settings: SchoolSettings | undefined
  loading: boolean
  error: unknown
  refresh: () => Promise<void>
}

const SettingsContext = createContext<SettingsContextValue | null>(null)

export function SettingsProvider({ children }: { children: ReactNode }) {
  const query = useQuery({
    queryKey: ["school-settings"],
    queryFn: getSchoolSettings,
    staleTime: Infinity,
    retry: false,
  })

  // Keep the money formatter in step with the settings. This runs before any
  // screen that displays an amount can paint, because the gate below waits for
  // the query to settle.
  useEffect(() => {
    if (query.data) {
      configureCurrency({
        code: query.data.currencyCode,
        symbol: query.data.currencySymbol,
      })
    }
  }, [query.data])

  const refresh = useCallback(async () => {
    await query.refetch()
  }, [query])

  const value = useMemo<SettingsContextValue>(
    () => ({
      settings: query.data,
      loading: query.isLoading,
      error: query.error,
      refresh,
    }),
    [query.data, query.isLoading, query.error, refresh],
  )

  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  )
}

export function useSchoolSettings(): SettingsContextValue {
  const context = useContext(SettingsContext)
  if (!context) {
    throw new Error("useSchoolSettings must be used inside <SettingsProvider>")
  }
  return context
}
