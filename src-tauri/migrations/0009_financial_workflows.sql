-- Allow a revoked discount grant to be issued again without permitting
-- duplicate active grants for the same scope.
DROP INDEX IF EXISTS uq_student_discount_all_terms;
CREATE UNIQUE INDEX uq_student_discount_all_terms
  ON student_discounts (student_id, discount_id)
  WHERE term_id IS NULL AND revoked_at IS NULL;

DROP INDEX IF EXISTS uq_student_discount_term;
CREATE UNIQUE INDEX uq_student_discount_term
  ON student_discounts (student_id, discount_id, term_id)
  WHERE term_id IS NOT NULL AND revoked_at IS NULL;
