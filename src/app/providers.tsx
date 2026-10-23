import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { useState, type ReactNode } from "react"
import { SessionProvider, SettingsProvider } from "./session"

/** Wires up TanStack Query and the session/settings contexts. */
export function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(
    () =>
      new QueryClient({
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
        <SettingsProvider>{children}</SettingsProvider>
      </SessionProvider>
    </QueryClientProvider>
  )
}
