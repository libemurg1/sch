-- Promotions keep the previous year's enrollment for history but mark it
-- INACTIVE, so a student has exactly one active enrollment at a time.
CREATE TABLE enrollments_new (
  id          TEXT PRIMARY KEY,
  student_id  TEXT NOT NULL REFERENCES students (id),
  class_id    TEXT NOT NULL REFERENCES classes (id),
  term_id     TEXT NOT NULL REFERENCES terms (id),
  status      TEXT NOT NULL DEFAULT 'ACTIVE'
              CHECK (status IN ('ACTIVE','INACTIVE','WITHDRAWN')),
  enrolled_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  UNIQUE (student_id, term_id)
);

INSERT INTO enrollments_new (id, student_id, class_id, term_id, status, enrolled_at)
SELECT id, student_id, class_id, term_id, status, enrolled_at FROM enrollments;

DROP TABLE enrollments;
ALTER TABLE enrollments_new RENAME TO enrollments;

CREATE INDEX idx_enrollments_class_term ON enrollments (class_id, term_id, status);
