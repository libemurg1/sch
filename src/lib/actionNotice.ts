export interface ActionNotice {
  message: string
}

const EVENT_NAME = "sfs:action-notice"

export function publishActionNotice(message: string): void {
  window.dispatchEvent(
    new CustomEvent<ActionNotice>(EVENT_NAME, { detail: { message } }),
  )
}

export function actionNoticeEventName(): string {
  return EVENT_NAME
}
