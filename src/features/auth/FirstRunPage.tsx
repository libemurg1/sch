import { useState, type FormEvent } from "react"
import { useSession } from "../../app/session"
import { Button, ErrorBanner, Field, Input } from "../../components"
import { AuthLayout } from "./AuthLayout"

/**
 * Shown once, on a brand-new install: there is no account yet, so the first
 * person through the door creates the administrator account.
 */
export function FirstRunPage() {
  const { bootstrap, status } = useSession()
  const [schoolName, setSchoolName] = useState("")
  const [displayName, setDisplayName] = useState("")
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [confirm, setConfirm] = useState("")
  const [localError, setLocalError] = useState<string | null>(null)

  const submit = (event: FormEvent) => {
    event.preventDefault()
    setLocalError(null)

    if (password.length < 8) {
      setLocalError("The password must be at least 8 characters.")
      return
    }
    if (password !== confirm) {
      setLocalError("The two passwords do not match.")
      return
    }

    bootstrap.mutate({
      displayName: displayName.trim(),
      username: username.trim(),
      password,
      role: "SCHOOL_ADMINISTRATOR",
      schoolName: schoolName.trim(),
    })
  }

  return (
    <AuthLayout subtitle="Welcome. Create the administrator account to get started.">
      {status?.sessionRestoreNotice ? (
        <p
          role="alert"
          className="mb-4 rounded-md bg-amber-50 px-3 py-2 text-sm text-amber-900 ring-1 ring-inset ring-amber-200"
        >
          {status.sessionRestoreNotice}
        </p>
      ) : null}
      <form onSubmit={submit} className="space-y-4">
        <Field
          label="School name"
          htmlFor="schoolName"
          hint="This is printed on invoices and receipts."
        >
          <Input
            id="schoolName"
            value={schoolName}
            onChange={(event) => setSchoolName(event.target.value)}
            placeholder="e.g. Riverside Academy"
            autoFocus
          />
        </Field>

        <Field label="Your name" htmlFor="displayName" required>
          <Input
            id="displayName"
            value={displayName}
            onChange={(event) => setDisplayName(event.target.value)}
            placeholder="e.g. Jane Banda"
            required
          />
        </Field>

        <Field
          label="Username"
          htmlFor="username"
          hint="Letters, numbers, dots, dashes and underscores."
          required
        >
          <Input
            id="username"
            value={username}
            onChange={(event) => setUsername(event.target.value)}
            autoComplete="username"
            minLength={3}
            required
          />
        </Field>

        <Field
          label="Password"
          htmlFor="password"
          hint="At least 8 characters."
          required
        >
          <Input
            id="password"
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete="new-password"
            minLength={8}
            required
          />
        </Field>

        <Field label="Confirm password" htmlFor="confirm" required>
          <Input
            id="confirm"
            type="password"
            value={confirm}
            onChange={(event) => setConfirm(event.target.value)}
            autoComplete="new-password"
            required
          />
        </Field>

        {localError ? (
          <p className="text-sm text-rose-600">{localError}</p>
        ) : null}
        {bootstrap.error ? <ErrorBanner error={bootstrap.error} /> : null}

        <Button
          type="submit"
          className="w-full"
          loading={bootstrap.isPending}
          disabled={
            displayName.trim() === "" ||
            username.trim() === "" ||
            password === "" ||
            confirm === ""
          }
        >
          Create account and sign in
        </Button>
      </form>
    </AuthLayout>
  )
}
