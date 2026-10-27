-- MVP safety: withdrawals, reversible financial corrections and audit support.

ALTER TABLE enrollments ADD COLUMN ended_at TEXT;
ALTER TABLE enrollments ADD COLUMN end_reason TEXT;

ALTER TABLE invoices ADD COLUMN cancelled_at TEXT;
ALTER TABLE invoices ADD COLUMN cancelled_by TEXT REFERENCES users (id);
ALTER TABLE invoices ADD COLUMN cancel_reason TEXT;

ALTER TABLE payments ADD COLUMN reversed_at TEXT;
ALTER TABLE payments ADD COLUMN reversed_by TEXT REFERENCES users (id);
ALTER TABLE payments ADD COLUMN reversal_reason TEXT;

ALTER TABLE payment_allocations ADD COLUMN reversed_at TEXT;
ALTER TABLE payment_allocations ADD COLUMN reversal_reason TEXT;

ALTER TABLE student_discounts ADD COLUMN revoked_at TEXT;
ALTER TABLE student_discounts ADD COLUMN revoked_by TEXT REFERENCES users (id);
ALTER TABLE student_discounts ADD COLUMN revoke_reason TEXT;

-- A reference can be reused after its payment has been reversed, but not while
-- an active payment still holds it.
DROP INDEX IF EXISTS uq_payments_reference;
CREATE UNIQUE INDEX uq_payments_reference
  ON payments (payment_method, reference_number)
  WHERE reference_number IS NOT NULL
    AND payment_method <> 'CASH'
    AND reversed_at IS NULL;

CREATE INDEX idx_payment_allocations_active
  ON payment_allocations (payment_id, reversed_at);
CREATE INDEX idx_student_discounts_active
  ON student_discounts (student_id, term_id, revoked_at);
