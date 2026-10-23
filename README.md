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

## Business Rules

The application is built around a few important financial rules:

- Money is handled in integer cents instead of floating point values.
- Invoices are generated from the fee structure snapshot at the time of billing.
- Payment allocations can be split across multiple invoices.
- Discounts may be fixed or percentage-based and are applied according to business logic in the backend.
- The database enforces local, transactional behavior for financial operations.
- Printing uses app-generated HTML layouts for structured documents.

## Database and Local Storage

The app stores data in a local SQLite database managed by the Rust backend. The database is initialized and migrated through SQL files in the `src-tauri/migrations` folder.

## Notes

- The app is intended for local/offline use, not a multi-user web deployment.
- Most financial operations are implemented in the Rust backend to keep business logic centralized and consistent.
- The frontend acts as a desktop UI layer over Tauri commands and the local database.

## License

This project does not currently declare a license in the repository. If you plan to distribute or reuse it, add an explicit license before publishing.

