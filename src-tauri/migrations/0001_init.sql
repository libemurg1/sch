PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- USERS (real accounts, attribution only, full access for all)
CREATE TABLE users (
  id            TEXT PRIMARY KEY,
  username      TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  display_name  TEXT NOT NULL,
  is_active     INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  last_login_at TEXT
);

-- PAYMENT METHODS (data-driven)
CREATE TABLE payment_methods (
  id         TEXT PRIMARY KEY,
  code       TEXT NOT NULL UNIQUE CHECK (code IN ('CASH','MOBILE_MONEY','BANK_TRANSFER','CHEQUE')),
  name       TEXT NOT NULL,
  is_active  INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
  sort_order INTEGER NOT NULL DEFAULT 0
);

INSERT INTO payment_methods (id, code, name, sort_order) VALUES
  (lower(hex(randomblob(16))), 'CASH', 'Cash', 0),
  (lower(hex(randomblob(16))), 'MOBILE_MONEY', 'Mobile Money', 1),
  (lower(hex(randomblob(16))), 'BANK_TRANSFER', 'Bank Transfer', 2),
  (lower(hex(randomblob(16))), 'CHEQUE', 'Cheque', 3);

CREATE TABLE students (
  id             TEXT PRIMARY KEY,
  student_number TEXT NOT NULL UNIQUE,
  first_name     TEXT NOT NULL CHECK (length(trim(first_name)) > 0),
  last_name      TEXT NOT NULL CHECK (length(trim(last_name)) > 0),
  dob            TEXT NOT NULL,
  gender         TEXT,
  admission_date TEXT NOT NULL,
  guardian_name  TEXT,
  guardian_phone TEXT,
  guardian_email TEXT,
  status         TEXT NOT NULL DEFAULT 'ACTIVE'
                 CHECK (status IN ('ACTIVE','INACTIVE','GRADUATED')),
  created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_students_name ON students (last_name, first_name);
CREATE INDEX idx_students_status ON students (status);

CREATE TABLE classes (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  grade_level INTEGER NOT NULL CHECK (grade_level >= 0),
  capacity    INTEGER NOT NULL CHECK (capacity > 0)
);

CREATE TABLE terms (
  id          TEXT PRIMARY KEY,
  year        INTEGER NOT NULL,
  term_number INTEGER NOT NULL CHECK (term_number BETWEEN 1 AND 4),
  start_date  TEXT NOT NULL,
  end_date    TEXT NOT NULL CHECK (end_date >= start_date),
  is_active   INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0,1)),
  UNIQUE (year, term_number)
);
CREATE UNIQUE INDEX uq_terms_one_active ON terms (is_active) WHERE is_active = 1;

CREATE TABLE enrollments (
  id          TEXT PRIMARY KEY,
  student_id  TEXT NOT NULL REFERENCES students (id),
  class_id    TEXT NOT NULL REFERENCES classes (id),
  term_id     TEXT NOT NULL REFERENCES terms (id),
  status      TEXT NOT NULL DEFAULT 'ACTIVE'
              CHECK (status IN ('ACTIVE','WITHDRAWN')),
  enrolled_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  UNIQUE (student_id, term_id)
);
CREATE INDEX idx_enrollments_class_term ON enrollments (class_id, term_id, status);

CREATE TABLE fee_structures (
  id          TEXT PRIMARY KEY,
  class_id    TEXT NOT NULL REFERENCES classes (id),
  term_id     TEXT NOT NULL REFERENCES terms (id),
  total_cents INTEGER NOT NULL CHECK (total_cents >= 0),
  UNIQUE (class_id, term_id)
);

CREATE TABLE fee_items (
  id               TEXT PRIMARY KEY,
  fee_structure_id TEXT NOT NULL REFERENCES fee_structures (id) ON DELETE CASCADE,
  description      TEXT NOT NULL CHECK (length(trim(description)) > 0),
  amount_cents     INTEGER NOT NULL CHECK (amount_cents > 0),
  discountable     INTEGER NOT NULL DEFAULT 1 CHECK (discountable IN (0,1)),
  sort_order       INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_fee_items_structure ON fee_items (fee_structure_id, sort_order);

CREATE TABLE discounts (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL UNIQUE CHECK (length(trim(name)) > 0),
  kind       TEXT NOT NULL CHECK (kind IN ('PERCENT','FIXED')),
  value      INTEGER NOT NULL CHECK (value > 0),
  is_active  INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  CHECK (kind <> 'PERCENT' OR value <= 10000)
);

CREATE TABLE student_discounts (
  id                   TEXT PRIMARY KEY,
  student_id           TEXT NOT NULL REFERENCES students (id),
  discount_id          TEXT NOT NULL REFERENCES discounts (id),
  term_id              TEXT REFERENCES terms (id),
  target_fee_item_id   TEXT REFERENCES fee_items (id),
  target_student_fee_id TEXT REFERENCES student_fees (id),
  reason               TEXT,
  created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  CHECK (target_fee_item_id IS NULL OR target_student_fee_id IS NULL)
);
CREATE UNIQUE INDEX uq_student_discount_all_terms
  ON student_discounts (student_id, discount_id) WHERE term_id IS NULL;
CREATE UNIQUE INDEX uq_student_discount_term
  ON student_discounts (student_id, discount_id, term_id) WHERE term_id IS NOT NULL;
CREATE INDEX idx_student_discounts_student ON student_discounts (student_id);

CREATE TABLE invoices (
  id               TEXT PRIMARY KEY,
  invoice_number   TEXT NOT NULL UNIQUE,
  kind             TEXT NOT NULL DEFAULT 'TERM' CHECK (kind IN ('TERM','SUPPLEMENTARY')),
  student_id       TEXT NOT NULL REFERENCES students (id),
  term_id          TEXT NOT NULL REFERENCES terms (id),
  fee_structure_id TEXT REFERENCES fee_structures (id),
  issue_date       TEXT NOT NULL,
  due_date         TEXT NOT NULL CHECK (due_date >= issue_date),
  status           TEXT NOT NULL DEFAULT 'UNPAID'
                   CHECK (status IN ('UNPAID','PARTIAL','PAID','CANCELLED')),
  subtotal_cents   INTEGER NOT NULL CHECK (subtotal_cents >= 0),
  discount_cents   INTEGER NOT NULL DEFAULT 0 CHECK (discount_cents >= 0),
  total_cents      INTEGER NOT NULL CHECK (total_cents >= 0),
  paid_cents       INTEGER NOT NULL DEFAULT 0,
  CHECK (discount_cents <= subtotal_cents),
  CHECK (total_cents = subtotal_cents - discount_cents),
  CHECK (paid_cents >= 0 AND paid_cents <= total_cents),
  CHECK (status <> 'PAID'    OR paid_cents = total_cents),
  CHECK (status <> 'UNPAID'  OR paid_cents = 0),
  CHECK (kind = 'SUPPLEMENTARY' OR fee_structure_id IS NOT NULL)
);
CREATE UNIQUE INDEX uq_invoices_student_term
  ON invoices (student_id, term_id) WHERE kind = 'TERM' AND status <> 'CANCELLED';
CREATE INDEX idx_invoices_term_status ON invoices (term_id, status);
CREATE INDEX idx_invoices_student ON invoices (student_id);

CREATE TABLE student_fees (
  id           TEXT PRIMARY KEY,
  student_id   TEXT NOT NULL REFERENCES students (id),
  term_id      TEXT NOT NULL REFERENCES terms (id),
  description  TEXT NOT NULL CHECK (length(trim(description)) > 0),
  amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
  discountable INTEGER NOT NULL DEFAULT 0 CHECK (discountable IN (0,1)),
  status       TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING','BILLED','REMOVED')),
  invoice_id   TEXT REFERENCES invoices (id),
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  CHECK ((status = 'BILLED') = (invoice_id IS NOT NULL))
);
CREATE INDEX idx_student_fees_student_term ON student_fees (student_id, term_id, status);

CREATE TABLE invoice_items (
  id           TEXT PRIMARY KEY,
  invoice_id   TEXT NOT NULL REFERENCES invoices (id) ON DELETE CASCADE,
  kind         TEXT NOT NULL CHECK (kind IN ('FEE','STUDENT_FEE','DISCOUNT')),
  description  TEXT NOT NULL,
  amount_cents INTEGER NOT NULL,
  discountable INTEGER NOT NULL DEFAULT 0 CHECK (discountable IN (0,1)),
  discount_id  TEXT REFERENCES discounts (id),
  sort_order   INTEGER NOT NULL DEFAULT 0,
  CHECK ((kind = 'DISCOUNT' AND amount_cents < 0)
      OR (kind <> 'DISCOUNT' AND amount_cents > 0))
);
CREATE INDEX idx_invoice_items_invoice ON invoice_items (invoice_id, sort_order);

CREATE TABLE payments (
  id               TEXT PRIMARY KEY,
  receipt_number   TEXT NOT NULL UNIQUE,
  student_id       TEXT NOT NULL REFERENCES students (id),
  amount_cents     INTEGER NOT NULL CHECK (amount_cents > 0),
  payment_date     TEXT NOT NULL,
  payment_method   TEXT NOT NULL,
  reference_number TEXT,
  created_by       TEXT REFERENCES users (id),
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  FOREIGN KEY (payment_method) REFERENCES payment_methods (code)
);
CREATE INDEX idx_payments_student ON payments (student_id, payment_date);
CREATE UNIQUE INDEX uq_payments_reference
  ON payments (payment_method, reference_number)
  WHERE reference_number IS NOT NULL AND payment_method <> 'CASH';

CREATE TABLE payment_allocations (
  id           TEXT PRIMARY KEY,
  payment_id   TEXT NOT NULL REFERENCES payments (id) ON DELETE CASCADE,
  invoice_id   TEXT NOT NULL REFERENCES invoices (id),
  amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
  UNIQUE (payment_id, invoice_id)
);
CREATE INDEX idx_payment_allocations_invoice ON payment_allocations (invoice_id);

CREATE TABLE number_counters (
  key        TEXT PRIMARY KEY,
  last_value INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE school_settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE audit_log (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  actor_id    TEXT REFERENCES users (id),
  entity      TEXT NOT NULL,
  entity_id   TEXT NOT NULL,
  action      TEXT NOT NULL,
  detail_json TEXT,
  at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_audit_entity ON audit_log (entity, entity_id);
CREATE INDEX idx_audit_actor ON audit_log (actor_id, at);
