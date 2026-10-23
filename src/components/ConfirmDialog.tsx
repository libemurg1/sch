import type { ReactNode } from "react"
import { Button } from "./Button"
import { ErrorBanner } from "./ErrorBanner"
import { Modal } from "./Modal"

export interface ConfirmDialogProps {
  open: boolean
  title: string
  children: ReactNode
  confirmLabel?: string
  loading?: boolean
  error?: unknown
  onCancel: () => void
  onConfirm: () => void
}

/** A small confirmation before a destructive action. */
export function ConfirmDialog({
  open,
  title,
  children,
  confirmLabel = "Delete",
  loading = false,
  error,
  onCancel,
  onConfirm,
}: ConfirmDialogProps) {
  return (
    <Modal
      open={open}
      onClose={onCancel}
      title={title}
      size="sm"
      footer={
        <>
          <Button variant="secondary" onClick={onCancel}>
            Cancel
          </Button>
          <Button variant="danger" onClick={onConfirm} loading={loading}>
            {confirmLabel}
          </Button>
        </>
      }
    >
      <div className="space-y-3">
        <div className="text-sm text-slate-600">{children}</div>
        {error ? <ErrorBanner error={error} /> : null}
      </div>
    </Modal>
  )
}
