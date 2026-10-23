import { useEffect, useRef, useState } from "react"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { setLogo, updateSchoolSettings } from "../../api/settings"
import { useSchoolSettings } from "../../app/session"
import { Page } from "../../app/AppShell"
import {
  Button,
  Card,
  ErrorBanner,
  Field,
  Input,
  PageHeader,
  Spinner,
  Textarea,
} from "../../components"
import { formatMoney } from "../../lib/money"
import type { SchoolSettings } from "../../types/schema/SchoolSettings"

interface FormState {
  schoolName: string
  address: string
  phone: string
  email: string
  currencyCode: string
  currencySymbol: string
  bankName: string
  bankAccountName: string
  bankAccountNumber: string
  bankBranch: string
  bankSwift: string
  invoiceFooter: string
  receiptFooter: string
}

function toForm(settings: SchoolSettings): FormState {
  return {
    schoolName: settings.schoolName,
    address: settings.address ?? "",
    phone: settings.phone ?? "",
    email: settings.email ?? "",
    currencyCode: settings.currencyCode,
    currencySymbol: settings.currencySymbol,
    bankName: settings.bankName ?? "",
    bankAccountName: settings.bankAccountName ?? "",
    bankAccountNumber: settings.bankAccountNumber ?? "",
    bankBranch: settings.bankBranch ?? "",
    bankSwift: settings.bankSwift ?? "",
    invoiceFooter: settings.invoiceFooter ?? "",
    receiptFooter: settings.receiptFooter ?? "",
  }
}

const emptyToNull = (value: string) => {
  const trimmed = value.trim()
  return trimmed === "" ? null : trimmed
}

const MAX_LOGO_BYTES = 1_500_000

/** School profile, branding, currency, banking details and document footers. */
export function SettingsPage() {
  const client = useQueryClient()
  const { settings } = useSchoolSettings()
  const [form, setForm] = useState<FormState | null>(null)
  const [saved, setSaved] = useState(false)
  const [logoError, setLogoError] = useState<string | null>(null)
  const fileInput = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (settings) {
      setForm(toForm(settings))
    }
  }, [settings])

  const update = useMutation({
    mutationFn: updateSchoolSettings,
    onSuccess: (updated) => {
      client.setQueryData(["school-settings"], updated)
      setSaved(true)
      window.setTimeout(() => setSaved(false), 2500)
    },
  })

  const logo = useMutation({
    mutationFn: setLogo,
    onSuccess: (updated) => {
      client.setQueryData(["school-settings"], updated)
      setLogoError(null)
    },
  })

  if (!form || !settings) {
    return (
      <Page>
        <PageHeader title="School Settings" />
        <div className="flex justify-center py-16">
          <Spinner className="h-6 w-6 text-slate-400" />
        </div>
      </Page>
    )
  }

  const set = <K extends keyof FormState>(key: K, value: FormState[K]) =>
    setForm((current) => (current ? { ...current, [key]: value } : current))

  const submit = () => {
    update.mutate({
      schoolName: form.schoolName.trim(),
      address: emptyToNull(form.address),
      phone: emptyToNull(form.phone),
      email: emptyToNull(form.email),
      currencyCode: form.currencyCode.trim(),
      currencySymbol: form.currencySymbol.trim(),
      bankName: emptyToNull(form.bankName),
      bankAccountName: emptyToNull(form.bankAccountName),
      bankAccountNumber: emptyToNull(form.bankAccountNumber),
      bankBranch: emptyToNull(form.bankBranch),
      bankSwift: emptyToNull(form.bankSwift),
      invoiceFooter: emptyToNull(form.invoiceFooter),
      receiptFooter: emptyToNull(form.receiptFooter),
    })
  }

  const onLogoSelected = (file: File | undefined) => {
    if (!file) {
      return
    }
    if (!file.type.startsWith("image/")) {
      setLogoError("Choose an image file (PNG, JPEG or WebP).")
      return
    }
    if (file.size > MAX_LOGO_BYTES) {
      setLogoError("The logo must be smaller than 1.5 MB.")
      return
    }

    const reader = new FileReader()
    reader.onload = () => {
      if (typeof reader.result === "string") {
        logo.mutate(reader.result)
      }
    }
    reader.onerror = () => setLogoError("The image could not be read.")
    reader.readAsDataURL(file)
  }

  return (
    <Page>
      <PageHeader
        title="School Settings"
        description="These details appear on invoices, statements and receipts."
        actions={
          <Button onClick={submit} loading={update.isPending}>
            Save changes
          </Button>
        }
      />

      {update.error ? <ErrorBanner error={update.error} /> : null}
      {saved ? (
        <div className="rounded-md bg-emerald-50 px-4 py-3 text-sm text-emerald-800 ring-1 ring-inset ring-emerald-200">
          Settings saved.
        </div>
      ) : null}

      <Card title="School profile" description="Your school's identity and contact details.">
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="School name" htmlFor="schoolName" required className="sm:col-span-2">
            <Input
              id="schoolName"
              value={form.schoolName}
              onChange={(event) => set("schoolName", event.target.value)}
              required
            />
          </Field>
          <Field label="Address" htmlFor="address" className="sm:col-span-2">
            <Textarea
              id="address"
              rows={2}
              value={form.address}
              onChange={(event) => set("address", event.target.value)}
              placeholder="Street, town, country"
            />
          </Field>
          <Field label="Phone" htmlFor="phone">
            <Input
              id="phone"
              value={form.phone}
              onChange={(event) => set("phone", event.target.value)}
            />
          </Field>
          <Field label="Email" htmlFor="email">
            <Input
              id="email"
              type="email"
              value={form.email}
              onChange={(event) => set("email", event.target.value)}
            />
          </Field>
        </div>
      </Card>

      <Card title="Logo" description="Shown in the app and at the top of printed documents.">
        <div className="flex flex-wrap items-center gap-4">
          <div className="flex h-20 w-20 items-center justify-center overflow-hidden rounded-md bg-slate-50 ring-1 ring-inset ring-slate-200">
            {settings.logoData ? (
              <img
                src={settings.logoData}
                alt="School logo"
                className="max-h-20 max-w-20 object-contain"
              />
            ) : (
              <span className="text-xs text-slate-400">No logo</span>
            )}
          </div>
          <div className="flex flex-col gap-2">
            <div className="flex gap-2">
              <Button
                variant="secondary"
                onClick={() => fileInput.current?.click()}
                loading={logo.isPending}
              >
                Choose image
              </Button>
              {settings.logoData ? (
                <Button
                  variant="ghost"
                  onClick={() => logo.mutate(null)}
                  disabled={logo.isPending}
                >
                  Remove
                </Button>
              ) : null}
            </div>
            <p className="text-xs text-slate-500">
              PNG, JPEG or WebP, up to 1.5 MB. A wide logo works best.
            </p>
          </div>
          <input
            ref={fileInput}
            type="file"
            accept="image/*"
            className="hidden"
            onChange={(event) => {
              onLogoSelected(event.target.files?.[0])
              event.target.value = ""
            }}
          />
        </div>
        {logoError ? <p className="mt-3 text-sm text-rose-600">{logoError}</p> : null}
        {logo.error ? <div className="mt-3"><ErrorBanner error={logo.error} /></div> : null}
      </Card>

      <Card
        title="Currency"
        description="How money is written throughout the app and on documents."
      >
        <div className="grid gap-4 sm:grid-cols-3">
          <Field label="Currency code" htmlFor="currencyCode" hint="e.g. KES, USD, ZAR">
            <Input
              id="currencyCode"
              value={form.currencyCode}
              onChange={(event) => set("currencyCode", event.target.value.toUpperCase())}
            />
          </Field>
          <Field label="Symbol" htmlFor="currencySymbol" hint="e.g. KSh, $, R">
            <Input
              id="currencySymbol"
              value={form.currencySymbol}
              onChange={(event) => set("currencySymbol", event.target.value)}
            />
          </Field>
          <Field label="Preview">
            <div className="rounded-md bg-slate-50 px-3 py-2 text-sm text-slate-700 ring-1 ring-inset ring-slate-200">
              {form.currencySymbol.trim() || "KSh"} 1,250.50
            </div>
          </Field>
        </div>
      </Card>

      <Card
        title="Banking details"
        description="Printed on invoices so families know where to pay."
      >
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="Bank name" htmlFor="bankName">
            <Input
              id="bankName"
              value={form.bankName}
              onChange={(event) => set("bankName", event.target.value)}
            />
          </Field>
          <Field label="Account name" htmlFor="bankAccountName">
            <Input
              id="bankAccountName"
              value={form.bankAccountName}
              onChange={(event) => set("bankAccountName", event.target.value)}
            />
          </Field>
          <Field label="Account number" htmlFor="bankAccountNumber">
            <Input
              id="bankAccountNumber"
              value={form.bankAccountNumber}
              onChange={(event) => set("bankAccountNumber", event.target.value)}
            />
          </Field>
          <Field label="Branch" htmlFor="bankBranch">
            <Input
              id="bankBranch"
              value={form.bankBranch}
              onChange={(event) => set("bankBranch", event.target.value)}
            />
          </Field>
          <Field label="SWIFT / sort code" htmlFor="bankSwift">
            <Input
              id="bankSwift"
              value={form.bankSwift}
              onChange={(event) => set("bankSwift", event.target.value)}
            />
          </Field>
        </div>
      </Card>

      <Card title="Document footers" description="Optional closing text on printed documents.">
        <div className="grid gap-4">
          <Field
            label="Invoice footer"
            htmlFor="invoiceFooter"
            hint="For example payment terms or a thank-you note."
          >
            <Textarea
              id="invoiceFooter"
              rows={2}
              value={form.invoiceFooter}
              onChange={(event) => set("invoiceFooter", event.target.value)}
            />
          </Field>
          <Field label="Receipt footer" htmlFor="receiptFooter">
            <Textarea
              id="receiptFooter"
              rows={2}
              value={form.receiptFooter}
              onChange={(event) => set("receiptFooter", event.target.value)}
            />
          </Field>
        </div>
      </Card>

      <div className="flex items-center justify-between rounded-lg bg-white px-4 py-3 shadow-sm ring-1 ring-slate-200">
        <p className="text-xs text-slate-500">
          Amounts currently display as{" "}
          <span className="font-medium text-slate-700">
            {formatMoney(125050)}
          </span>
          .
        </p>
        <Button onClick={submit} loading={update.isPending}>
          Save changes
        </Button>
      </div>
    </Page>
  )
}
