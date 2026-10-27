-- GRADES: the fixed Zimbabwean structure. Nursery and ECD sit below Grade 1.
CREATE TABLE grades (
  level      INTEGER PRIMARY KEY CHECK (level BETWEEN -2 AND 7),
  name       TEXT NOT NULL UNIQUE,
  sort_order INTEGER NOT NULL UNIQUE
);

INSERT INTO grades (level, name, sort_order) VALUES
  (-2, 'Nursery', 0),
  (-1, 'ECD A', 1),
  (0, 'ECD B', 2),
  (1, 'Grade 1', 3),
  (2, 'Grade 2', 4),
  (3, 'Grade 3', 5),
  (4, 'Grade 4', 6),
  (5, 'Grade 5', 7),
  (6, 'Grade 6', 8),
  (7, 'Grade 7', 9);

-- CLASSES: one set of classes per academic year. A class is a grade plus an
-- optional free-text stream ("Blue", "A", ...). The name is derived by the
-- application from the grade and stream. Existing classes are kept and
-- assigned to the active (or latest) year.
PRAGMA legacy_alter_table = ON;
ALTER TABLE classes RENAME TO classes_old;

CREATE TABLE classes (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  grade_level   INTEGER NOT NULL REFERENCES grades (level),
  stream        TEXT NOT NULL DEFAULT '',
  academic_year INTEGER NOT NULL,
  capacity      INTEGER NOT NULL CHECK (capacity > 0)
);

-- Old builds stored Nursery, ECD A and ECD B all as grade_level 0, so the new
-- level is read from the name for those rows. Unknown level-0 names fall back
-- to Nursery. The name itself is normalised for the ECD seeds.
INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity)
SELECT
  id,
  CASE
    WHEN UPPER(TRIM(name)) IN ('ECD-A', 'ECD A') THEN 'ECD A'
    WHEN UPPER(TRIM(name)) IN ('ECD-B', 'ECD B') THEN 'ECD B'
    ELSE name
  END,
  CASE
    WHEN grade_level <> 0 THEN grade_level
    WHEN UPPER(name) LIKE 'ECD%A%' OR UPPER(name) LIKE 'ECD A%' THEN -1
    WHEN UPPER(name) LIKE 'ECD%B%' OR UPPER(name) LIKE 'ECD B%' THEN 0
    ELSE -2
  END,
  TRIM(
    SUBSTR(
      name,
      LENGTH(
        CASE
          WHEN grade_level <> 0 THEN 'Grade ' || grade_level
          WHEN UPPER(name) LIKE 'NURSERY%' THEN 'Nursery'
          WHEN UPPER(name) LIKE 'ECD%A%' OR UPPER(name) LIKE 'ECD A%' THEN 'ECD A'
          WHEN UPPER(name) LIKE 'ECD%B%' OR UPPER(name) LIKE 'ECD B%' THEN 'ECD B'
          ELSE 'Nursery'
        END
      ) + 1
    ),
    ' -'
  ),
  COALESCE(
    (SELECT year FROM terms WHERE is_active = 1 LIMIT 1),
    (SELECT MAX(year) FROM terms),
    CAST(strftime('%Y', 'now') AS INTEGER)
  ),
  capacity
FROM classes_old;

DROP TABLE classes_old;

CREATE UNIQUE INDEX uq_classes_year_name ON classes (academic_year, name);
CREATE INDEX idx_classes_year_grade ON classes (academic_year, grade_level);

PRAGMA legacy_alter_table = OFF;

-- PROMOTIONS: one row per student per year they move into. Keeps the decision
-- (promoted, repeated or graduated) and the class they moved to.
CREATE TABLE promotions (
  id            TEXT PRIMARY KEY,
  student_id    TEXT NOT NULL REFERENCES students (id),
  from_year     INTEGER NOT NULL,
  to_year       INTEGER NOT NULL,
  from_class_id TEXT NOT NULL REFERENCES classes (id),
  to_class_id   TEXT REFERENCES classes (id),
  outcome       TEXT NOT NULL CHECK (outcome IN ('PROMOTED','REPEATED','GRADUATED')),
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  created_by    TEXT REFERENCES users (id),
  UNIQUE (student_id, to_year),
  CHECK ((outcome = 'GRADUATED') = (to_class_id IS NULL))
);

CREATE INDEX idx_promotions_student ON promotions (student_id, to_year);
