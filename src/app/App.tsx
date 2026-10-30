import { HashRouter, Navigate, Route, Routes } from "react-router-dom"
import { AppShell } from "./AppShell"
import { Providers } from "./providers"
import { useSchoolSettings, useSession } from "./session"
import { FirstRunPage } from "../features/auth/FirstRunPage"
import { LoginPage } from "../features/auth/LoginPage"
import { Button, ErrorBanner, Spinner } from "../components"
import { DashboardPage } from "../features/dashboard/DashboardPage"
import { StudentsPage } from "../features/students/StudentsPage"
import { StudentDetailPage } from "../features/students/StudentDetailPage"
import { EnrollmentPage } from "../features/enrollment/EnrollmentPage"
import { ClassesPage } from "../features/enrollment/ClassesPage"
import { TermsPage } from "../features/enrollment/TermsPage"
import { PromotionPage } from "../features/enrollment/PromotionPage"
import { FeeStructuresPage } from "../features/feeStructures/FeeStructuresPage"
import { DiscountsPage } from "../features/discounts/DiscountsPage"
import { BillingPage } from "../features/billing/BillingPage"
import { InvoicesPage } from "../features/billing/InvoicesPage"
import { InvoiceDetailPage } from "../features/billing/InvoiceDetailPage"
import { PaymentsPage } from "../features/payments/PaymentsPage"
import { StatementsPage } from "../features/statements/StatementsPage"
import { MaintenancePage } from "../features/maintenance/MaintenancePage"
import { SettingsPage } from "../features/settings/SettingsPage"
import { UsersPage } from "../features/settings/UsersPage"
import { ReportsPage } from "../features/reports/ReportsPage"

/**
 * The router uses hashes because the app is served from Tauri's own protocol,
 * where a path based route would break on reload.
 */
export default function App() {
  return (
    <Providers>
      <HashRouter>
        <AuthGate>
          <Routes>
            <Route element={<AppShell />}>
              <Route index element={<DashboardPage />} />
              <Route path="students" element={<StudentsPage />} />
              <Route path="students/:studentId" element={<StudentDetailPage />} />
              <Route path="classes" element={<ClassesPage />} />
              <Route path="terms" element={<TermsPage />} />
              <Route path="enrollment" element={<EnrollmentPage />} />
              <Route path="promotion" element={<PromotionPage />} />
              <Route path="fee-structures" element={<FeeStructuresPage />} />
              <Route path="discounts" element={<DiscountsPage />} />
              <Route path="billing" element={<BillingPage />} />
              <Route path="invoices" element={<InvoicesPage />} />
              <Route
                path="billing/invoices/:invoiceId"
                element={<InvoiceDetailPage />}
              />
              <Route path="payments" element={<PaymentsPage />} />
              <Route path="statements" element={<StatementsPage />} />
              <Route path="reports" element={<ReportsPage />} />
              <Route
                path="settings"
                element={
                  <AdministratorOnly>
                    <SettingsPage />
                  </AdministratorOnly>
                }
              />
              <Route
                path="users"
                element={
                  <AdministratorOnly>
                    <UsersPage />
                  </AdministratorOnly>
                }
              />
              <Route
                path="maintenance"
                element={
                  <AdministratorOnly>
                    <MaintenancePage />
                  </AdministratorOnly>
                }
              />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Route>
          </Routes>
        </AuthGate>
      </HashRouter>
    </Providers>
  )
}

function AdministratorOnly({ children }: { children: React.ReactNode }) {
  const { user } = useSession()
  return user?.role === "SCHOOL_ADMINISTRATOR" ? (
    <>{children}</>
  ) : (
    <Navigate to="/" replace />
  )
}

/**
 * Nothing renders until we know whether the install has an account and who is
 * signed in, so no screen can flash data before the login screen appears.
 */
function AuthGate({ children }: { children: React.ReactNode }) {
  const session = useSession()
  const settings = useSchoolSettings()

  if (session.loading || settings.loading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-slate-100">
        <Spinner className="h-6 w-6 text-slate-400" />
      </div>
    )
  }

  if (session.error) {
    return (
      <div className="mx-auto flex min-h-screen max-w-md flex-col justify-center px-4">
        <ErrorBanner error={session.error} onRetry={session.refresh} />
        <div className="mt-4">
          <Button variant="secondary" onClick={() => window.location.reload()}>
            Reload the app
          </Button>
        </div>
      </div>
    )
  }

  if (!session.hasUsers) {
    return <FirstRunPage />
  }

  if (!session.user) {
    return <LoginPage />
  }

  return <>{children}</>
}
