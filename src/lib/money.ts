/**
 * Money is an integer number of cents everywhere between the database and the
 * UI (see README "Core Principles"). This module is the only place that turns
 * cents into something a human reads, or something a human typed back into
 * cents. It never uses floating point arithmetic, so no rounding error can
 * creep into a total.
 */

/**
 * The currency the school bills in. The backend is deliberately
 * currency-agnostic - it only stores cents - so the currency is a school
 * setting applied here at runtime by `SettingsProvider`.
 */
export interface Currency {
  code: string
  symbol: string
}

export const DEFAULT_CURRENCY: Currency = {
  code: "USD",
  symbol: "$",
}

let activeCurrency: Currency = DEFAULT_CURRENCY

/** Applies the currency from the school settings to every later format call. */
export function configureCurrency(currency?: Partial<Currency> | null) {
  const code = currency?.code?.trim()
  const symbol = currency?.symbol?.trim()
  activeCurrency = {
    code: code && code.length > 0 ? code : DEFAULT_CURRENCY.code,
    symbol: symbol && symbol.length > 0 ? symbol : DEFAULT_CURRENCY.symbol,
  }
}

/** The currency currently applied by `configureCurrency`. */
export function currentCurrency(): Currency {
  return activeCurrency
}

/** Formats integer cents as a plain decimal string, e.g. `125050` -> `1,250.50`. */
export function formatCents(cents: number): string {
  const negative = cents < 0
  const absolute = Math.abs(Math.trunc(cents))
  const whole = Math.floor(absolute / 100)
  const fraction = absolute % 100
  return `${negative ? "-" : ""}${groupThousands(whole)}.${String(fraction).padStart(2, "0")}`
}

/** Formats integer cents with the currency symbol, e.g. `KSh 1,250.50`. */
export function formatMoney(cents: number): string {
  const negative = cents < 0
  const sign = negative ? "-" : ""
  return `${sign}${activeCurrency.symbol} ${formatCents(Math.abs(cents))}`
}

/**
 * Parses money typed by a person into integer cents.
 *
 * Accepts `1250`, `1250.5`, `1,250.50`, `KSh 1,250.50` and `1 250`. Anything
 * with more than one decimal point or any other stray characters is rejected.
 * Returns `null` when the input is not a usable amount, so a form can show a
 * validation message instead of silently billing zero.
 */
export function parseMoneyToCents(input: string): number | null {
  const cleaned = input.trim().replace(/[^0-9.\-]/g, "")
  if (cleaned === "" || cleaned === "-" || cleaned === "." || cleaned === "-.") {
    return null
  }

  const negative = cleaned.startsWith("-")
  const digits = negative ? cleaned.slice(1) : cleaned
  if (!/^\d*(\.\d*)?$/.test(digits)) {
    return null
  }

  const [wholeRaw = "", fractionRaw = ""] = digits.split(".")
  const whole = wholeRaw === "" ? 0 : Number(wholeRaw)
  if (!Number.isSafeInteger(whole)) {
    return null
  }

  // Normalise the fraction to exactly two digits, rounding half up, without
  // ever going through a float.
  let fraction = Number((fractionRaw + "00").slice(0, 2))
  if (fractionRaw.length > 2 && Number(fractionRaw[2]) >= 5) {
    fraction += 1
  }
  if (fraction >= 100) {
    // 0.999 rounds up into the next whole unit.
    const carried = whole + 1
    if (!Number.isSafeInteger(carried)) {
      return null
    }
    return negative ? -carried * 100 : carried * 100
  }

  const cents = whole * 100 + fraction
  if (!Number.isSafeInteger(cents)) {
    return null
  }
  return negative ? -cents : cents
}

/** Parses a plain integer count of cents, used for raw cent inputs. */
export function parseCents(input: string): number | null {
  const cleaned = input.trim().replace(/[^0-9\-]/g, "")
  if (cleaned === "" || cleaned === "-" || !/^-?\d+$/.test(cleaned)) {
    return null
  }
  const value = Number(cleaned)
  return Number.isSafeInteger(value) ? value : null
}

/**
 * Converts a percentage a person typed (`10`, `10.5`) into the basis points the
 * backend stores (`1000`, `1050`).
 */
export function parsePercentToBasisPoints(input: string): number | null {
  const cleaned = input.trim().replace(/[^0-9.]/g, "")
  if (cleaned === "" || !/^\d*(\.\d*)?$/.test(cleaned)) {
    return null
  }
  const [wholeRaw = "", fractionRaw = ""] = cleaned.split(".")
  const whole = wholeRaw === "" ? 0 : Number(wholeRaw)
  let fraction = Number((fractionRaw + "00").slice(0, 2))
  if (fractionRaw.length > 2 && Number(fractionRaw[2]) >= 5) {
    fraction += 1
  }
  const basisPoints = whole * 100 + fraction
  return Number.isSafeInteger(basisPoints) ? basisPoints : null
}

/** Renders basis points as a percentage for display, e.g. `1000` -> `10%`. */
export function formatBasisPoints(basisPoints: number): string {
  const whole = Math.trunc(basisPoints) / 100
  const remainder = Math.trunc(basisPoints) % 100
  if (remainder === 0) {
    return `${whole}%`
  }
  return `${whole}.${String(remainder).padStart(2, "0")}%`
}

/** Renders cents as an editable decimal string, e.g. `125050` -> `1250.50`. */
export function centsToInputValue(cents: number): string {
  return formatCents(cents)
}

/** Adds up a list of cent amounts. */
export function sumCents(amounts: number[]): number {
  return amounts.reduce((total, amount) => total + amount, 0)
}

/** Groups the integer part of a number with thousands separators. */
export function groupThousands(value: number): string {
  return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, ",")
}

/** Formats a count with thousands separators. */
export function formatCount(value: number): string {
  return groupThousands(Math.trunc(value))
}
