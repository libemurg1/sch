# School Fees System

A Tauri-based desktop application for managing school fees, student records, class enrollments, invoices, discounts, and payments in a local-first offline environment.

## Overview

This project combines a React + TypeScript frontend with a Rust backend and SQLite database to provide a school finance workflow for:

- registering and managing students
- setting up academic classes and terms
- defining fee structures
- generating term and supplementary invoices
- recording student payments and allocations
- applying discounts and viewing statements
- printing receipts, invoices, and account statements

The app is designed to run locally on a desktop machine and keep sensitive financial data on-device.

## Tech Stack

- Frontend: React 19, TypeScript, Vite, TailwindCSS
- State/data access: TanStack Query, Zod, Tauri invoke commands
- Backend: Rust, Tauri v2
- Database: SQLite via rusqlite with bundled migration support
- Printing: browser print dialog / HTML print layout

## Project Structure

```text
.
├─ src/
│  ├─ api/                  # Tauri command wrappers and client helpers
│  ├─ app/                  # app shell, session, providers, routing
│  ├─ components/           # shared UI components
│  ├─ features/             # feature modules (auth, billing, payments, students, etc.)
│  ├─ lib/                  # utility helpers, money/date logic
│  ├─ reports/              # invoice/receipt/statement print documents
│  ├─ types/                # TypeScript types and schema outputs
│  ├─ index.css             # global styling
│  └─ main.tsx              # app bootstrap
├─ src-tauri/
│  ├─ bindings/             # generated TypeScript interfaces from Rust
│  ├─ capabilities/         # Tauri capability configuration
│  ├─ migrations/           # SQL migration files
│  ├─ src/
│  │  ├─ application/       # service layer/use cases
│  │  ├─ commands/          # Tauri command handlers
│  │  ├─ domain/            # entities, rules, errors, auth, money logic
│  │  ├─ infrastructure/    # DB access, repositories, utilities
│  │  └─ main.rs            # app bootstrap and command registration
│  ├─ Cargo.toml            # Rust project configuration
│  └─ tauri.conf.json       # Tauri app config
├─ index.html
├─ package.json
├─ tailwind.config.js
├─ postcss.config.js
├─ tsconfig.json
├─ vite.config.ts
└─ README.md
```

## Prerequisites

Before running the app, install:

- Node.js 18+ or 20+ (LTS recommended)
- Rust toolchain via `rustup`
- Tauri prerequisites for your OS
  - Windows: Visual Studio 2022 Build Tools + WebView2 runtime
  - Windows release builds: Perl (used to build SQLCipher's vendored OpenSSL)

## Getting Started

Install dependencies:

```bash
npm install
```

Start the app in development mode:

```bash
npm run tauri dev
```

For a frontend-only build check:

```bash
npm run build
```

To create a production desktop build:

```bash
npm run tauri build
```

Run the frontend checks before building:

```bash
npm test
npm run lint
npm run typecheck
npm audit
```

The Windows CI workflow runs those frontend checks and the Rust test suite on
pushes and pull requests.

## Offline license issuer

The desktop app verifies signed offline license keys. It contains only the
public verification key; the private signing key is stored separately by the
software provider and must never be copied into the app or committed to source
control.

Build the private issuer utility from a Windows development machine with Rust:

```powershell
cargo build --release --manifest-path tools/license-issuer/Cargo.toml
```

Create the signing key once, if it has not already been initialized. By
default, the utility stores it under the current Windows user's local
application data and protects it with DPAPI:

```powershell
tools\license-issuer\target\release\sfs-license-issuer.exe init
```

The command prints the public verification key. It is already initialized for
this source tree; use `public-key` to display it again:

```powershell
tools\license-issuer\target\release\sfs-license-issuer.exe public-key
```

The key in `src-tauri/license-public-key.txt` must match that value; the issuer
refuses to create keys if the private key does not match the app's trusted
public key. Do not initialize a second signing key unless you also plan a
signed-app update that trusts its new public key; existing installed versions
will only trust the public key they were built with.

Create and store an encrypted, portable backup of the private signing key.
The utility prompts for a passphrase (minimum 12 characters):

```powershell
tools\license-issuer\target\release\sfs-license-issuer.exe export-key `
  E:\SecureBackups\sfs-signing-key.sfskey
```

Keep that file and its passphrase separately and securely. To restore it on a
replacement issuer computer, copy the backup there and run:

```powershell
tools\license-issuer\target\release\sfs-license-issuer.exe import-key `
  E:\SecureBackups\sfs-signing-key.sfskey
```

To issue an annual key, ask the school for its activation ID from
**System > Maintenance > License**, then run:

```powershell
tools\license-issuer\target\release\sfs-license-issuer.exe issue `
  --activation-id "DEVICE-ACTIVATION-ID" `
  --school "Example School" `
  --starts-on 2026-10-27 `
  --expires-on 2027-10-26
```

Send the resulting `sfs1...` key to that school. The key is bound to the
activation ID and will not activate another device. The issuer is an offline
tool; it does not contact a server. Consequently, keys cannot be remotely
revoked and device transfers require provider intervention. The app allows a
14-day first-launch trial, a five-day grace period after trial/license expiry,
then permits read-only access and backups.

The issuer does not process payments or maintain a customer subscription
database. Maintain a separate, access-controlled customer/license register
with school, activation ID, payment status, start/expiry dates, and any
replacement-device decisions. Since licensing is offline, the rule not to
replace a device before expiry is an issuer operating policy, not a remotely
enforced safeguard.

## Building a Windows Installer (.exe or .msi)

This project is configured as a Tauri desktop app, so the normal production build can be packaged into Windows installer artifacts.

### Prerequisites for Windows packaging

Install the following before generating an installer:

- Node.js and npm
- Rust toolchain
- Visual Studio 2022 Build Tools with:
  - Desktop development with C++
  - MSVC toolchain
- WebView2 runtime

### Generate the installer

From the project root:

```bash
npm run tauri build
```

This creates the bundled release output in the Tauri target folder, typically under:

```text
src-tauri/target/release/bundle/
```

Depending on the build environment and configuration, you will usually find:

- Windows installer executable: `.exe`
- Windows MSI installer: `.msi`

### Use a single target only

If you want only one installer type instead of all default outputs, set the bundle target explicitly in `src-tauri/tauri.conf.json`:

```json
"bundle": {
  "active": true,
  "targets": ["nsis", "msi"]
}
```

or:

```json
"bundle": {
  "active": true,
  "targets": "nsis"
}
```

Then run:

```bash
npm run tauri build
```

### Common output locations

Typical Windows output locations are:

```text
src-tauri/target/release/bundle/nsis/
src-tauri/target/release/bundle/msi/
```

- `nsis` usually contains a `.exe` installer
- `msi` usually contains a `.msi` installer

### Installing the generated app

After the build completes:

1. Open the generated installer in the relevant bundle folder.
2. Follow the setup wizard.
3. Install the app on the target Windows machine.

### For distribution

For professional deployment, a signed installer is recommended for enterprise distribution. You can sign the generated `.exe` or `.msi` payload with a trusted code-signing certificate before shipping it to users.

## Typical Workflow

1. Create users and configure school settings.
2. Add classes and academic terms.
3. Register students and enroll them into classes.
4. Define fee structures and assign fees.
5. Generate term invoices or supplemental invoices.
6. Record payments and allocate them against invoices.
7. Print receipts, statements, or invoice documents as needed.
8. Review collections, outstanding balances, overdue invoices, and the audit
   trail in Finance Reports.
9. Create a database backup and export a copy to a removable or external drive.

## Business Rules

The application is built around a few important financial rules:

- Money is handled in integer cents instead of floating point values.
- Invoices are generated from the fee structure snapshot at the time of billing.
- Payment allocations can be split across multiple invoices.
- Discounts may be fixed or percentage-based and are applied according to business logic in the backend.
- The database enforces local, transactional behavior for financial operations.
- Printing uses app-generated HTML layouts for structured documents.
- Student statements include an account ledger: invoices are debits; payments
  and invoice cancellations are credits; payment reversals are debits. Each row
  shows the running balance. Term-filtered statements include payments
  allocated to that term.
- Financial corrections are recorded as reversals or cancellations with a
  required reason; the original transaction remains in the history.
- Finance reports summarize collections by payment method for a selected date
  range and show outstanding and overdue balances by term.
- Financial events are written to the audit trail with the signed-in user and
  are viewable in Finance Reports.

## Database and Local Storage

The app stores data in a local SQLite database managed by the Rust backend. The database is initialized and migrated through SQL files in the `src-tauri/migrations` folder.

On Windows, the live database and local snapshots use SQLCipher encryption.
The database key is protected by Windows DPAPI for the current Windows user;
local `.db` snapshots are therefore intended to be restored under that same
Windows user profile. The first launch after upgrading encrypts the existing
database and valid local snapshots, taking a migration snapshot before
upgrading the live database.

Use Maintenance to export a portable `.sfsbackup` file for another computer or
Windows account. Portable exports are encrypted by SQLCipher using a
passphrase of at least 12 characters. The passphrase is not stored by the
application; losing it makes that export unrecoverable. Copies of old,
pre-encryption `.db` backups on removable drives were not migrated
automatically and should be treated as unencrypted.

## Notes

- The app is intended for local/offline use, not a multi-user web deployment.
- The initial operating model is one trusted operator with full access. The app
  does not implement role-based permissions; any additional active account
  also has full access.
- Backups are initially stored beside the app's local data. Use Maintenance to
  export a passphrase-protected copy to an existing folder on a removable or
  external drive. Keep at least one copy separate from the computer that runs
  the app, and keep its passphrase separately.
- Database encryption currently depends on the Windows user profile. Protect
  that profile and the machine with Windows account controls and full-disk
  encryption; use portable encrypted exports for disaster recovery.
- Most financial operations are implemented in the Rust backend to keep business logic centralized and consistent.
- The frontend acts as a desktop UI layer over Tauri commands and the local database.

## License

This project does not currently declare a license in the repository. If you plan to distribute or reuse it, add an explicit license before publishing.
