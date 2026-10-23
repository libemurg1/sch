import { useEffect, useState, type FormEvent } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  changePassword,
  createUser,
  deleteUser,
  listUsers,
  setUserActive,
  updateUser,
} from "../../api/auth"
import { useSession } from "../../app/session"
import { Page } from "../../app/AppShell"
import {
  Badge,
  Button,
  Card,
  ConfirmDialog,
  ErrorBanner,
  Field,
  Input,
  Modal,
  PageHeader,
  Table,
  type Column,
} from "../../components"
import { formatDateTime } from "../../lib/dates"
import type { User } from "../../types/schema/User"

/** Account management: add colleagues, rename them, deactivate them, change a password. */
export function UsersPage() {
  const client = useQueryClient()
  const { user: currentUser } = useSession()
  const users = useQuery({ queryKey: ["users"], queryFn: listUsers })

  const [adding, setAdding] = useState(false)
  const [editing, setEditing] = useState<User | null>(null)
  const [deleting, setDeleting] = useState<User | null>(null)

  const toggleActive = useMutation({
    mutationFn: ({ user, active }: { user: User; active: boolean }) =>
      setUserActive(user.id, active),
    onSuccess: () => client.invalidateQueries({ queryKey: ["users"] }),
  })

  const remove = useMutation({
    mutationFn: (target: User) => deleteUser(target.id),
    onSuccess: () => {
      client.invalidateQueries()
      setDeleting(null)
    },
  })

  const columns: Column<User>[] = [
    {
      key: "name",
      header: "Name",
      render: (user) => (
        <div>
          <p className="font-medium text-slate-900">{user.displayName}</p>
          {user.id === currentUser?.id ? (
            <p className="text-xs text-indigo-600">You</p>
          ) : null}
        </div>
      ),
    },
    {
      key: "username",
      header: "Username",
      render: (user) => <span className="font-mono text-xs">{user.username}</span>,
    },
    {
      key: "status",
      header: "Status",
      render: (user) => (
        <Badge tone={user.isActive ? "success" : "danger"}>
          {user.isActive ? "Active" : "Deactivated"}
        </Badge>
      ),
    },
    {
      key: "lastLogin",
      header: "Last sign-in",
      hideOnSmall: true,
      render: (user) =>
        user.lastLoginAt ? (
          <span className="text-xs text-slate-500">
            {formatDateTime(user.lastLoginAt)}
          </span>
        ) : (
          <span className="text-xs text-slate-400">Never</span>
        ),
    },
    {
      key: "actions",
      header: "",
      align: "right",
      render: (user) => (
        <div className="flex flex-wrap justify-end gap-2">
          <Button
            size="sm"
            variant="secondary"
            onClick={() => setEditing(user)}
          >
            Edit
          </Button>
          {user.id === currentUser?.id ? null : (
            <>
              <Button
                size="sm"
                variant={user.isActive ? "danger" : "secondary"}
                loading={toggleActive.isPending}
                onClick={() =>
                  toggleActive.mutate({ user, active: !user.isActive })
                }
              >
                {user.isActive ? "Deactivate" : "Reactivate"}
              </Button>
              <Button
                size="sm"
                variant="ghost"
                className="text-rose-600 hover:bg-rose-50 hover:text-rose-700"
                onClick={() => setDeleting(user)}
              >
                Delete
              </Button>
            </>
          )}
        </div>
      ),
    },
  ]

  return (
    <Page>
      <PageHeader
        title="Users"
        description="Everyone who can sign in to this app."
        actions={<Button onClick={() => setAdding(true)}>Add user</Button>}
      />

      {users.error ? <ErrorBanner error={users.error} onRetry={() => users.refetch()} /> : null}
      {toggleActive.error ? <ErrorBanner error={toggleActive.error} /> : null}

      <Card
        title="Accounts"
        description="Payments record who received them, so give each person their own account."
      >
        <Table
          columns={columns}
          rows={users.data ?? []}
          keyOf={(user) => user.id}
          empty={<p className="text-sm text-slate-500">No accounts yet.</p>}
        />
      </Card>

      <ChangePasswordCard />

      <AddUserModal
        open={adding}
        onClose={() => setAdding(false)}
        onCreated={() => {
          setAdding(false)
          client.invalidateQueries({ queryKey: ["users"] })
        }}
      />

      <EditUserModal
        user={editing}
        onClose={() => setEditing(null)}
        onSaved={async (updated) => {
          await client.invalidateQueries({ queryKey: ["users"] })
          if (updated.id === currentUser?.id) {
            await client.refetchQueries({ queryKey: ["auth"] })
          }
          setEditing(null)
        }}
      />

      <ConfirmDialog
        open={deleting !== null}
        title="Delete user"
        loading={remove.isPending}
        error={remove.error}
        onCancel={() => {
          remove.reset()
          setDeleting(null)
        }}
        onConfirm={() => {
          if (deleting) {
            remove.mutate(deleting)
          }
        }}
      >
        <p>
          Delete <strong>{deleting?.displayName}</strong>? They will no longer be
          able to sign in.
        </p>
        <p className="mt-2">
          Payments they recorded stay in the system, but lose the received-by
          name.
        </p>
      </ConfirmDialog>
    </Page>
  )
}


function EditUserModal({
  user,
  onClose,
  onSaved,
}: {
  user: User | null
  onClose: () => void
  onSaved: (user: User) => void
}) {
  const [displayName, setDisplayName] = useState("")
  const [username, setUsername] = useState("")
  const [localError, setLocalError] = useState<string | null>(null)

  useEffect(() => {
    setDisplayName(user?.displayName ?? "")
    setUsername(user?.username ?? "")
    setLocalError(null)
  }, [user])

  const mutation = useMutation({
    mutationFn: () =>
      updateUser(user?.id as string, username.trim(), displayName.trim()),
    onSuccess: onSaved,
  })

  const submit = (event: FormEvent) => {
    event.preventDefault()
    setLocalError(null)
    const name = displayName.trim()
    const login = username.trim()
    if (name === "") {
      setLocalError("The name cannot be empty.")
      return
    }
    if (name.length > 80) {
      setLocalError("The name must be 80 characters or fewer.")
      return
    }
    if (!/^[A-Za-z0-9._-]{3,32}$/.test(login)) {
      setLocalError(
        "The username must be 3 to 32 letters, numbers, dots, dashes or underscores.",
      )
      return
    }
    if (name === user?.displayName && login === user?.username) {
      onClose()
      return
    }
    mutation.mutate()
  }

  return (
    <Modal
      open={user !== null}
      onClose={onClose}
      title="Edit user"
      description="Change the name shown in the app and on receipts, or the sign-in username."
    >
      <form onSubmit={submit} className="space-y-4">
        <Field label="Full name" htmlFor="editDisplayName" required>
          <Input
            id="editDisplayName"
            value={displayName}
            onChange={(event) => setDisplayName(event.target.value)}
            maxLength={80}
            required
          />
        </Field>
        <Field
          label="Username"
          htmlFor="editUsername"
          hint="Letters, numbers, dots, dashes and underscores."
          required
        >
          <Input
            id="editUsername"
            value={username}
            onChange={(event) => setUsername(event.target.value)}
            minLength={3}
            maxLength={32}
            required
          />
        </Field>

        {localError ? (
          <p className="text-sm text-rose-600">{localError}</p>
        ) : null}
        {mutation.error ? <ErrorBanner error={mutation.error} /> : null}

        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="submit"
            loading={mutation.isPending}
            disabled={
              displayName.trim() === "" || username.trim() === ""
            }
          >
            Save changes
          </Button>
        </div>
      </form>
    </Modal>
  )
}

function ChangePasswordCard() {
  const [current, setCurrent] = useState("")
  const [next, setNext] = useState("")
  const [confirm, setConfirm] = useState("")
  const [localError, setLocalError] = useState<string | null>(null)
  const [done, setDone] = useState(false)

  const mutation = useMutation({
    mutationFn: changePassword,
    onSuccess: () => {
      setCurrent("")
      setNext("")
      setConfirm("")
      setDone(true)
      window.setTimeout(() => setDone(false), 2500)
    },
  })

  const submit = (event: FormEvent) => {
    event.preventDefault()
    setLocalError(null)
    if (next.length < 8) {
      setLocalError("The new password must be at least 8 characters.")
      return
    }
    if (next !== confirm) {
      setLocalError("The two new passwords do not match.")
      return
    }
    mutation.mutate({ currentPassword: current, newPassword: next })
  }

  return (
    <Card title="Change your password">
      <form onSubmit={submit} className="grid gap-4 sm:grid-cols-3">
        <Field label="Current password" htmlFor="currentPassword" required>
          <Input
            id="currentPassword"
            type="password"
            value={current}
            onChange={(event) => setCurrent(event.target.value)}
            autoComplete="current-password"
            required
          />
        </Field>
        <Field label="New password" htmlFor="newPassword" required>
          <Input
            id="newPassword"
            type="password"
            value={next}
            onChange={(event) => setNext(event.target.value)}
            autoComplete="new-password"
            minLength={8}
            required
          />
        </Field>
        <Field label="Confirm new password" htmlFor="confirmPassword" required>
          <Input
            id="confirmPassword"
            type="password"
            value={confirm}
            onChange={(event) => setConfirm(event.target.value)}
            autoComplete="new-password"
            required
          />
        </Field>

        <div className="sm:col-span-3">
          {localError ? (
            <p className="mb-2 text-sm text-rose-600">{localError}</p>
          ) : null}
          {mutation.error ? (
            <div className="mb-2">
              <ErrorBanner error={mutation.error} />
            </div>
          ) : null}
          {done ? (
            <p className="mb-2 text-sm text-emerald-700">
              Your password has been changed.
            </p>
          ) : null}
          <Button
            type="submit"
            loading={mutation.isPending}
            disabled={current === "" || next === "" || confirm === ""}
          >
            Update password
          </Button>
        </div>
      </form>
    </Card>
  )
}

function AddUserModal({
  open,
  onClose,
  onCreated,
}: {
  open: boolean
  onClose: () => void
  onCreated: () => void
}) {
  const [displayName, setDisplayName] = useState("")
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [confirm, setConfirm] = useState("")
  const [localError, setLocalError] = useState<string | null>(null)

  const mutation = useMutation({
    mutationFn: createUser,
    onSuccess: () => {
      setDisplayName("")
      setUsername("")
      setPassword("")
      setConfirm("")
      onCreated()
    },
  })

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
    mutation.mutate({
      displayName: displayName.trim(),
      username: username.trim(),
      password,
    })
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Add a user"
      description="They can sign in with this username and password."
    >
      <form onSubmit={submit} className="space-y-4">
        <Field label="Full name" htmlFor="newDisplayName" required>
          <Input
            id="newDisplayName"
            value={displayName}
            onChange={(event) => setDisplayName(event.target.value)}
            required
          />
        </Field>
        <Field
          label="Username"
          htmlFor="newUsername"
          hint="Letters, numbers, dots, dashes and underscores."
          required
        >
          <Input
            id="newUsername"
            value={username}
            onChange={(event) => setUsername(event.target.value)}
            minLength={3}
            required
          />
        </Field>
        <Field label="Password" htmlFor="newUserPassword" required>
          <Input
            id="newUserPassword"
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete="new-password"
            minLength={8}
            required
          />
        </Field>
        <Field label="Confirm password" htmlFor="newUserConfirm" required>
          <Input
            id="newUserConfirm"
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
        {mutation.error ? <ErrorBanner error={mutation.error} /> : null}

        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="submit"
            loading={mutation.isPending}
            disabled={
              displayName.trim() === "" ||
              username.trim() === "" ||
              password === "" ||
              confirm === ""
            }
          >
            Add user
          </Button>
        </div>
      </form>
    </Modal>
  )
}
