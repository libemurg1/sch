import {
  MutationCache,
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query"
import { useState, type ReactNode } from "react"
import { SessionProvider, SettingsProvider } from "./session"
import { ActionNotifications } from "../components/ActionNotifications"
import { publishActionNotice } from "../lib/actionNotice"

/** Wires up TanStack Query and the session/settings contexts. */
export function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(
    () =>
      new QueryClient({
        mutationCache: new MutationCache({
          onSuccess: (_data, _variables, _context, mutation) => {
            if (mutation.options.meta?.showSuccessNotice === false) {
              return
            }
            const message = mutation.options.meta?.successMessage
            const resolvedMessage =
              typeof message === "function"
                ? message(_data, _variables)
                : message
            publishActionNotice(
              typeof resolvedMessage === "string"
                ? resolvedMessage
                : "Action completed successfully.",
            )
          },
        }),
        defaultOptions: {
          queries: {
            // The database only changes when this app changes it, so refetching
            // on every window focus would be wasted work.
            staleTime: 30_000,
            refetchOnWindowFocus: false,
            retry: 1,
          },
        },
      }),
  )

  return (
    <QueryClientProvider client={client}>
      <SessionProvider>
        <SettingsProvider>
          {children}
          <ActionNotifications />
        </SettingsProvider>
      </SessionProvider>
    </QueryClientProvider>
  )
}
