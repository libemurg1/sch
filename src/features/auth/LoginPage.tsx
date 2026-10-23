import { useState, type FormEvent } from "react"
import { useSession } from "../../app/session"
import { Button, ErrorBanner, Field, Input } from "../../components"
import { AuthLayout } from "./AuthLayout"

/** The sign-in screen. Shown whenever nobody is signed in. */
export function LoginPage() {
  const { login } = useSession()
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")

  const submit = (event: FormEvent) => {
    event.preventDefault()
    login.mutate({ username: username.trim(), password })
  }

  return (
    <AuthLayout subtitle="Sign in to continue">
      <form onSubmit={submit} className="space-y-4">
        <Field label="Username" htmlFor="username" required>
          <Input
            id="username"
            value={username}
            onChange={(event) => setUsername(event.target.value)}
            autoComplete="username"
            autoFocus
            required
          />
        </Field>

        <Field label="Password" htmlFor="password" required>
          <Input
            id="password"
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete="current-password"
            required
          />
        </Field>

        {login.error ? <ErrorBanner error={login.error} /> : null}

        <Button
          type="submit"
          className="w-full"
          loading={login.isPending}
          disabled={username.trim() === "" || password === ""}
        >
          Sign in
        </Button>
      </form>
    </AuthLayout>
  )
}
