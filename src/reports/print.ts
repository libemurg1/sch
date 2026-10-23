/**
 * Printing goes through WebView2's system print dialog, so "Save as PDF" is
 * available for free and the browser does not stamp the document URL, date and
 * page number on the page (see README "Printing").
 *
 * A receipt has to come out on an 80 mm thermal roll while everything else
 * comes out on A4 or Letter. The page size is controlled by an `@page` rule,
 * which has to be in the document at print time, so the rule is injected just
 * before printing and left in place for the next document.
 */
import { invoke } from "@tauri-apps/api/core"

const STYLE_ID = "sfs-print-page-size"

const A4_RULE = `@page { size: A4 portrait; margin: 12mm; }`
const LETTER_RULE = `@page { size: Letter portrait; margin: 12mm; }`
const RECEIPT_RULE = `@page { size: 80mm auto; margin: 2mm; }`

export type PaperSize = "a4" | "letter" | "receipt"

/** Prints the current page, optionally on a specific paper size. */
export function printDocument(paper: PaperSize = "a4") {
  const existing = document.getElementById(STYLE_ID)
  existing?.remove()

  const rule =
    paper === "receipt" ? RECEIPT_RULE : paper === "letter" ? LETTER_RULE : A4_RULE

  const style = document.createElement("style")
  style.id = STYLE_ID
  style.textContent = rule
  document.head.appendChild(style)

  // Give the browser a moment to apply the new page rules.
  window.setTimeout(async () => {
    try {
      await invoke("show_print_dialog")
    } catch {
      // Older WebView2 runtimes do not expose the system dialog.
      window.print()
    }
  }, 50)
}
