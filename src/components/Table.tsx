import type { ReactNode } from "react"
import { cx } from "../lib/cx"
import { EmptyState } from "./EmptyState"

export interface Column<T> {
  /** Identifies the column for alignment and styling. */
  key: string
  header: ReactNode
  render: (row: T) => ReactNode
  align?: "left" | "center" | "right"
  /** Hides the column on small screens. */
  hideOnSmall?: boolean
  className?: string
}

export interface TableProps<T> {
  columns: Column<T>[]
  rows: T[]
  keyOf: (row: T) => string
  empty?: ReactNode
  onRowClick?: (row: T) => void
  className?: string
}

const ALIGN: Record<NonNullable<Column<unknown>["align"]>, string> = {
  left: "text-left",
  center: "text-center",
  right: "text-right",
}

/** A plain data table. */
export function Table<T>({
  columns,
  rows,
  keyOf,
  empty,
  onRowClick,
  className,
}: TableProps<T>) {
  if (rows.length === 0) {
    return <>{empty ?? <EmptyState title="Nothing here yet" />}</>
  }

  return (
    <div className={cx("-mx-4 overflow-x-auto sm:-mx-5", className)}>
      <table className="min-w-full divide-y divide-slate-200 text-sm">
        <thead>
          <tr>
            {columns.map((column) => (
              <th
                key={column.key}
                scope="col"
                className={cx(
                  "px-4 py-2.5 text-xs font-semibold uppercase tracking-wide text-slate-500 sm:px-5",
                  ALIGN[column.align ?? "left"],
                  column.hideOnSmall ? "hidden md:table-cell" : null,
                  column.className,
                )}
              >
                {column.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100 bg-white">
          {rows.map((row) => (
            <tr
              key={keyOf(row)}
              onClick={onRowClick ? () => onRowClick(row) : undefined}
              className={cx(
                onRowClick ? "cursor-pointer hover:bg-slate-50" : null,
              )}
            >
              {columns.map((column) => (
                <td
                  key={column.key}
                  className={cx(
                    "px-4 py-2.5 text-slate-700 sm:px-5",
                    ALIGN[column.align ?? "left"],
                    column.hideOnSmall ? "hidden md:table-cell" : null,
                    column.className,
                  )}
                >
                  {column.render(row)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
