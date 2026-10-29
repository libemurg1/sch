import { useState, type FormEvent } from "react"
import { useMutation } from "@tanstack/react-query"
import { recoverAdministrator } from "../../api/auth"
import { useSession } from "../../app/session"
import { Button, ErrorBanner, Field, Input } from "../../components"
import { AuthLayout } from "./AuthLayout"

/** The sign-in screen. Shown whenever nobody is signed in. */
export function LoginPage() {
  const { login, status } = useSession()
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [rememberMe, setRememberMe] = useState(false)
  const [recovering, setRecovering] = useState(false)
  const [recoveryUsername, setRecoveryUsername] = useState("")
  const [recoveryCode, setRecoveryCode] = useState("")
  const [newPassword, setNewPassword] = useState("")
  const [confirmNewPassword, setConfirmNewPassword] = useState("")
  const [recoveryError, setRecoveryError] = useState<string | null>(null)
  const recovery = useMutation({
    mutationFn: () =>
      recoverAdministrator(recoveryUsername.trim(), recoveryCode, newPassword),
  })

  const submit = (event: FormEvent) => {
    event.preventDefault()
    login.mutate({ username: username.trim(), password, rememberMe })
  }

  const submitRecovery = (event: FormEvent) => {
    event.preventDefault()
    setRecoveryError(null)
    if (newPassword.length < 8) {
      setRecoveryError("The new password must be at least 8 characters.")
      return
    }
    if (newPassword !== confirmNewPassword) {
      setRecoveryError("The two new passwords do not match.")
      return
    }
    recovery.mutate()
  }

  if (recovery.isSuccess) {
    return (
      <AuthLayout subtitle="Administrator password recovered">
        <div className="space-y-4">
          <p className="text-sm text-slate-700">
            Your password has been changed. A new one-time recovery code has
            replaced the code you just used. Save this code somewhere private
            and separate from this computer.
          </p>
          <Field
            label="New one-time recovery code — save it now"
            htmlFor="replacement-recovery-code"
          >
            <Input
              id="replacement-recovery-code"
              value={recovery.data}
              readOnly
              onFocus={(event) => event.currentTarget.select()}
              className="font-mono tracking-wider"
            />
          </Field>
          <Button
            className="w-full"
            onClick={() => {
              recovery.reset()
              setRecovering(false)
              setUsername(recoveryUsername)
              setPassword(newPassword)
            }}
          >
            Continue to sign in
          </Button>
        </div>
      </AuthLayout>
    )
  }

  return (
    <AuthLayout
      subtitle={
        recovering
          ? "Recover the sole School Administrator account"
          : "Sign in to continue"
      }
    >
      {recovering ? (
        <form onSubmit={submitRecovery} className="space-y-4">
          <p className="text-sm text-slate-600">
            Recovery works only if a recovery code was generated beforehand
            and you are the sole active School Administrator.
          </p>
          <Field label="Administrator username" htmlFor="recovery-username" required>
            <Input
              id="recovery-username"
              value={recoveryUsername}
              onChange={(event) => setRecoveryUsername(event.target.value)}
              autoComplete="username"
              required
            />
          </Field>
          <Field label="One-time recovery code" htmlFor="recovery-code" required>
            <Input
              id="recovery-code"
              value={recoveryCode}
              onChange={(event) => setRecoveryCode(event.target.value)}
              autoComplete="off"
              spellCheck={false}
              required
            />
          </Field>
          <Field
            label="New password"
            htmlFor="new-password"
            hint="At least 8 characters."
            required
          >
            <Input
              id="new-password"
              type="password"
              value={newPassword}
              onChange={(event) => setNewPassword(event.target.value)}
              autoComplete="new-password"
              minLength={8}
              required
            />
          </Field>
          <Field
            label="Confirm new password"
            htmlFor="confirm-new-password"
            required
          >
            <Input
              id="confirm-new-password"
              type="password"
              value={confirmNewPassword}
              onChange={(event) => setConfirmNewPassword(event.target.value)}
              autoComplete="new-password"
              minLength={8}
              required
            />
          </Field>
          {recoveryError ? (
            <p className="text-sm text-rose-600">{recoveryError}</p>
          ) : null}
          {recovery.error ? <ErrorBanner error={recovery.error} /> : null}
          <Button
            type="submit"
            className="w-full"
            loading={recovery.isPending}
            disabled={
              !recoveryUsername.trim() ||
              !recoveryCode.trim() ||
              !newPassword ||
              !confirmNewPassword
            }
          >
            Reset password
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="w-full"
            onClick={() => {
              setRecovering(false)
              setRecoveryError(null)
              recovery.reset()
            }}
          >
            Back to sign in
          </Button>
        </form>
      ) : (
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

        <label className="flex items-start gap-2 text-sm text-slate-600">
          <input
            type="checkbox"
            checked={rememberMe}
            onChange={(event) => setRememberMe(event.target.checked)}
            className="mt-0.5 rounded border-slate-300 text-indigo-600 focus:ring-indigo-600"
          />
          <span>
            Remember me on this Windows account
            <span className="mt-0.5 block text-xs text-slate-500">
              Stay signed in until you sign out or change your password. Avoid
              this on a shared Windows account.
            </span>
          </span>
        </label>

        {status?.sessionRestoreNotice ? (
          <p role="status" className="text-sm text-amber-700">
            {status.sessionRestoreNotice}
          </p>
        ) : null}

        {login.error ? <ErrorBanner error={login.error} /> : null}

        <Button
          type="submit"
          className="w-full"
          loading={login.isPending}
          disabled={username.trim() === "" || password === ""}
        >
          Sign in
        </Button>
        {status?.hasUsers ? (
          <Button
            type="button"
            variant="secondary"
            className="w-full"
            onClick={() => setRecovering(true)}
          >
            Forgot administrator password?
          </Button>
        ) : null}
      </form>
      )}
    </AuthLayout>
  )
}
