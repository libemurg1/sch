INSERT OR IGNORE INTO classes (id, name, grade_level, capacity) VALUES
  (lower(hex(randomblob(16))), 'Nursery', 0, 30),
  (lower(hex(randomblob(16))), 'ECD-A', 0, 30),
  (lower(hex(randomblob(16))), 'ECD-B', 0, 30),
  (lower(hex(randomblob(16))), 'Grade 1', 1, 30),
  (lower(hex(randomblob(16))), 'Grade 2', 2, 30),
  (lower(hex(randomblob(16))), 'Grade 3', 3, 30),
  (lower(hex(randomblob(16))), 'Grade 4', 4, 30),
  (lower(hex(randomblob(16))), 'Grade 5', 5, 30),
  (lower(hex(randomblob(16))), 'Grade 6', 6, 30),
  (lower(hex(randomblob(16))), 'Grade 7', 7, 30);