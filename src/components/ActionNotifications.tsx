import { useEffect, useState } from "react"
import {
  actionNoticeEventName,
  type ActionNotice,
} from "../lib/actionNotice"

export function ActionNotifications() {
  const [message, setMessage] = useState("")

  useEffect(() => {
    let timeout: number | undefined
    const handleNotice = (event: Event) => {
      const notice = (event as CustomEvent<ActionNotice>).detail
      setMessage(notice.message)
      window.clearTimeout(timeout)
      timeout = window.setTimeout(() => setMessage(""), 4500)
    }

    window.addEventListener(actionNoticeEventName(), handleNotice)
    return () => {
      window.removeEventListener(actionNoticeEventName(), handleNotice)
      window.clearTimeout(timeout)
    }
  }, [])

  if (!message) {
    return null
  }

  return (
    <div
      className="fixed right-4 top-4 z-[70] max-w-sm rounded-md bg-emerald-800 px-4 py-3 text-sm font-medium text-white shadow-lg"
      role="status"
      aria-live="polite"
    >
      {message}
    </div>
  )
}
