# School Fees System User Guide

## Install the application

Use the Windows installer provided by your school or system administrator.
Choose either the `.msi` package or the `.exe` setup package; you do not need
both. Open the installer and follow its prompts, then launch **Primary School**
from the Start menu or desktop shortcut.

### Requirements

- A supported 64-bit Windows computer.
- Microsoft Edge WebView2 Runtime. It is already present on many Windows
  computers. If it is missing, the current MSI and EXE installers download and
  install it, so the computer needs an internet connection during installation.
- No Node.js, Rust, Perl, or Visual Studio Build Tools are needed on the
  computer where the app is installed; those are build tools for developers.

For an offline installation, install the WebView2 Runtime on the target
computer before running the app installer. The default installers do not
include the full offline runtime.

## Sign-in and first launch

There is **no default admin username or password** built into the application.
On a new installation, the first-run screen asks you to create the administrator
account. You choose the username and password; `admin` is only an option for a
username, not a preset account.

On first launch:

1. Enter the school name. This is used on printed documents and can be changed
   later in Settings.
2. Enter your name and choose a username. Usernames must be 3–32 characters
   and may contain letters, numbers, dots, dashes, and underscores.
3. Choose and confirm a password of at least 8 characters.
4. Select **Create account and sign in**.

On later launches, sign in with the username and password you created. Usernames
are not case-sensitive. Keep your password private. There is no built-in
password reset if you forget it; change it while signed in from **System >
Users**.

## Typical setup and daily workflow

Use the left navigation to open each area. On a smaller screen, open the menu
with the menu button in the header.

### 1. Configure the school and academic year

- Open **System > Settings** to check the school name and other school details
  used by the app and printed documents.
- Open **Records > Terms** to create the academic year terms and set the term
  you are working in as active.
- Open **Records > Classes** to create the classes, grade levels, and capacities
  used for enrollment.

The active term is shown in the header. Check it before enrolling students,
setting fees, or billing.

### 2. Add students and enroll them

- In **Records > Students**, add each student and keep their details current.
- In **Records > Enrollment**, enroll students into the appropriate class and
  term.
- Use **Promotion** when moving students into the next class or term. Review
  the target class and term before confirming.

### 3. Set fees and discounts

- In **Records > Fee Structures**, configure the fees for the relevant class
  and term.
- In **Records > Discounts**, create discounts and assign them to eligible
  students or fees as appropriate.

Review fee amounts and discount eligibility before generating invoices.

### 4. Bill students

- Open **Money > Billing** to generate invoices for the selected term and
  eligible students.
- Open **Money > Invoices** to find invoices and review their details and
  payment status.
- Print an invoice from its detail view when needed.

Invoices preserve the fee amounts used when they were created. If an invoice
needs correction, use the cancellation action and enter a clear reason rather
than trying to change the historical transaction.

### 5. Record payments

- Open **Money > Payments** and record the payer, amount, date, and payment
  method.
- Allocate the payment to the appropriate invoice or invoices, checking the
  allocation amounts before saving.
- Print or reprint a receipt from the payment record.

If a payment was entered incorrectly, use the reversal action and provide a
reason. Reversals and cancellations preserve an audit trail; they are not the
same as deleting the financial history.

### 6. Review student accounts and reports

- Open **Money > Statements** to view a student's account ledger. Invoices and
  charges appear as debits; payments and invoice cancellations appear as
  credits; payment reversals appear as debits. The statement shows a running
  balance.
- Print a statement when needed, and confirm the student and term filters
  before printing.
- Open **Money > Reports** to review collections, balances, overdue fees, and
  the audit trail.
- Use the **Dashboard** for a summary of current activity.

## Users and access

The first account is the administrator account. The **System > Users** page
allows signed-in users to add accounts, edit account names, deactivate or
reactivate other accounts, delete accounts, and change their own password.
Each account has full application access; the current version does not provide
role-based restrictions. Only create accounts for people who are trusted to
view and manage all student and financial records.

Payments record which signed-in user received them. Deleting a user does not
delete their recorded payments, but removes that user's name from the
received-by attribution.

Select **Sign out** when you are finished, especially on a shared Windows
computer.

## License and activation

A new installation starts a 14-day trial at first launch. During setup, the
app creates a device activation ID. To purchase or renew a license, send that
ID to your software provider and provide the school name. The provider returns
a signed license key; enter it under **System > Maintenance > License**.

The app validates the key locally and does not need an internet connection.
Licenses cover one device and contain a start and expiry date. If the license
expires, a five-day grace period allows normal work while you arrange renewal.
After the grace period the app becomes read-only. You can continue viewing data
and making backups or portable exports, but cannot create or change school or
financial records.

The activation ID is specific to this installation. A license key issued for
another device will not activate. The current offline system cannot remotely
revoke keys or automatically transfer a license to a replacement computer.
Contact the software provider about replacement devices; under the current
policy a replacement key is issued after the existing license expires.

Subscription purchase and renewal are arranged directly with the software
provider; the app does not process payments.

The license uses the computer's date and protected local state. Deliberately
changing or resetting the computer or its Windows account can interfere with
offline license checks. Keep regular portable backups and the backup
passphrase; licensing does not replace data backups.

## Backups and recovery

Create backups regularly and before significant changes:

1. Open **System > Maintenance** and select **Back up now** to create an
   encrypted local backup.
2. For protection against computer or Windows-account loss, export a backup
   to an external or removable drive. Enter an existing destination folder,
   choose a passphrase of at least 12 characters, confirm it, and select
   **Export** for the backup.
3. Store the portable backup and its passphrase separately. The passphrase
   cannot be recovered if lost.

Local `.db` backups are protected for the Windows account that created them.
They are not a substitute for an off-device portable export when moving to
another Windows account or computer. Portable `.sfsbackup` files require the
passphrase used during export.

To restore a backup, use **System > Maintenance** and select a listed backup or
enter the path to a backup from another location. Enter the passphrase when
restoring a portable backup. **Restoring replaces the current database** with
the contents of the selected backup; records added after that backup will be
lost. Create a fresh backup first and verify the selected file before
confirming.

## Important operating notes

- The application is designed for local use on Windows and works without a
  cloud account. Keep the computer and its Windows account access protected.
- Financial records use a ledger with a running balance. Check student,
  invoice, payment method, and amount carefully before saving.
- Use cancellation and reversal actions with accurate reasons to correct
  financial records; do not treat them as ordinary deletions.
- Keep at least one usable portable backup on a separate device and make sure
  the recovery passphrase is available to the responsible school administrator.
- If the app is moved to another Windows account or computer, use a portable
  `.sfsbackup` export and its passphrase. The protected live database key and
  local backups are tied to the original Windows account.
