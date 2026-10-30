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
are not case-sensitive. Keep your password private. While signed in, change it
from **System > Users**. In **System > Maintenance**, generate an administrator
recovery code and store it somewhere private and separate from the computer.
The app stores only a hash and displays the code once; generating another code
invalidates the previous one. If the sole active School Administrator is locked
out, select **Forgot administrator password?** on the sign-in screen and use the
username and recovery code to set a new password. Recovery also generates a
replacement code, which must be saved. Recovery is rate-limited and is available
only when there is exactly one active School Administrator. It cannot help if
the code was never saved or the app database and all backups are lost.

On the login screen, **Remember me on this Windows account** keeps you signed in
across app restarts until you sign out or change your password. The app stores a
revocable session token protected for the current Windows account, not your
password. Do not select it on a shared Windows account or a computer other
people can access while you are signed in.

## Typical setup and daily workflow

Use the left navigation to open each area. On a smaller screen, open the menu
with the menu button in the header.

Successful changes show a confirmation message. Important financial actions
and sensitive exports ask you to review the details before proceeding; actions
that cannot be undone, such as cancelling an invoice or reversing a payment,
require a confirmation.

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
- Use **Find a student** to search the permanent student directory across all
  years by name, student number, or guardian details. Open a student record to
  see their full academic history, invoices, and payments.
- School Administrators can use **Import students** to preview and validate a
  CSV file before importing student details. Download the template for the
  supported headers; student numbers are assigned automatically, and existing
  numbers in an exported file are skipped. Imports do not create enrollments.
- **Export filtered students** downloads the currently filtered directory as
  CSV, including student numbers, dates of birth, and guardian contact details.
  Confirm the export and store the file securely.
- Use **Class roster by term** to view the students grouped into classes for a
  selected school year and term. Check the year and term shown before using a
  roster; students who were not enrolled in that term are listed separately.
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

- Before billing a class, record any student-specific charges under
  **Money > Billing**. A pending charge for a student in the selected class
  and term is included on that student's term invoice, not on other students'
  invoices. Then generate invoices for the class and term.
- If the student's class invoice has already been issued, record the charge
  and raise a supplementary invoice for that student. Issued invoices are not
  edited, so a printed copy remains consistent with the stored invoice.
- Open **Money > Invoices** to find invoices and review their details and
  payment status.
- Print an invoice from its detail view when needed.

Invoices preserve the fee amounts used when they were created. If an invoice
needs correction, use the cancellation action and enter a clear reason rather
than trying to change the historical transaction.

### 5. Record payments

- Open **Money > Payments** and record the payer, amount, date, and payment
  method. A reference is optional; when supplied for a non-cash payment, it
  must be unique for that payment method.
- The app applies the payment to the student's oldest outstanding invoices
  first. Any amount left after those balances are cleared remains as account
  credit for later invoices.
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
is available to School Administrators, who can add accounts, assign roles,
edit access, deactivate or reactivate accounts, and change their own password.

| Capability | School Administrator | Bursar |
|---|---|---|
| View students, invoices, payments, statements, reports and audit history | Allowed | Allowed |
| Register or edit student details | Allowed | Not allowed |
| Manage classes, terms, enrollment and promotion | Allowed | Allowed |
| Edit fee structures | Allowed | Allowed |
| Create and assign discounts | Allowed | Allowed |
| Raise class and supplementary invoices | Allowed | Allowed |
| Record payments and print receipts | Allowed | Allowed |
| Reverse payments | Allowed | Not allowed |
| Cancel invoices | Allowed | Not allowed |
| Manage users, school settings, licenses and backups | Allowed | Not allowed |

Use the **Bursar** role for finance staff who need to maintain fees, discounts,
enrollment, billing, and payment records. Use **School Administrator** only
for trusted staff who need unrestricted access. The backend checks permissions;
hiding a screen or button is not the security control.

Payments record which signed-in user received them. Deactivate accounts when
staff leave. The system prevents deactivating, deleting, or demoting the last
active School Administrator.

Select **Sign out** when you are finished, especially on a shared Windows
computer.

## License and activation

A new installation starts a 14-day trial at first launch (the app records the
first-launch date as the trial start date). The remaining days and trial start
and end dates are shown in the app banner and under **System > Maintenance >
License**. During setup, the app creates a device activation ID. To purchase or renew a license, send that
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
   encrypted local backup. Use **Database health > Check now** to confirm the
   database integrity and verify that the newest local backup is readable.
2. For protection against computer or Windows-account loss, export a backup
   to an external or removable drive. Enter an existing destination folder,
   choose a passphrase of at least 12 characters, confirm it, and select
   **Export** for the backup.
3. Store the portable backup and its passphrase separately. The passphrase
   cannot be recovered if lost.

Run the health check after backups and restores, and periodically test a
portable backup on a separate computer or Windows account. A successful local
backup protects against some database failures, but not against loss of the
computer, drive, or Windows account. Keep a recent encrypted portable backup
off-device.

If the database cannot be opened and no usable local backup exists, the app
preserves the old database files and starts an empty recovery database with a
warning. Do not enter new records into it. Create a temporary administrator,
then restore an off-device `.sfsbackup` from **System > Maintenance**. Keep the
archived files until the restored data has been checked.

Local `.db` backups are protected for the Windows account that created them.
They are not a substitute for an off-device portable export when moving to
another Windows account or computer. Portable `.sfsbackup` files require the
passphrase used during export.

To restore a backup, use **System > Maintenance** and select a listed backup or
enter the path to a backup from another location. Enter the passphrase when
restoring a portable backup. **Restoring replaces the current database** with
the contents of the selected backup; records added after that backup will be
lost. The app creates a safety snapshot before restoring and checks database
integrity and record relationships afterwards. If restore or migration fails,
it attempts to reinstate the original database and reports whether rollback
succeeded. Create a fresh backup first and verify the selected file before
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
