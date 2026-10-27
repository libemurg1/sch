import { useEffect, useState } from "react"
import { useClasses, useMoveStudent } from "../../api/queries"
import { Button, ErrorBanner, Field, Modal, Select } from "../../components"

export function MoveStudentModal({
  open,
  onClose,
  studentId,
  studentLabel,
  termId,
  termYear,
  currentClassId,
}: {
  open: boolean
  onClose: () => void
  studentId: string
  studentLabel: string
  termId: string
  termYear: number
  currentClassId: string
}) {
  const classes = useClasses(termYear)
  const move = useMoveStudent()
  const [classId, setClassId] = useState(currentClassId)

  useEffect(() => {
    if (open) {
      setClassId(currentClassId)
      move.reset()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, currentClassId])

  const submit = () => {
    move.mutate(
      { studentId, termId, classId },
      { onSuccess: onClose },
    )
  }

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Move student"
      description={`Change the class of ${studentLabel} for this term only.`}
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button
            onClick={submit}
            loading={move.isPending}
            disabled={!classId || classId === currentClassId}
          >
            Move student
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {move.isError ? <ErrorBanner error={move.error} /> : null}
        <Field label="Class" htmlFor="move-class" required>
          <Select
            id="move-class"
            value={classId}
            onChange={(event) => setClassId(event.target.value)}
          >
            {(classes.data ?? []).map((klass) => (
              <option key={klass.id} value={klass.id}>
                {klass.name}
                {klass.stream ? ` (${klass.stream})` : ""} · capacity {klass.capacity}
              </option>
            ))}
          </Select>
        </Field>
        <p className="text-xs text-slate-500">
          Fee structures and invoices already raised stay with their original
          class.
        </p>
      </div>
    </Modal>
  )
}
