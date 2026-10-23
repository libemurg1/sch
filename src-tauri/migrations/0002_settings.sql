-- School configuration (branding, currency, contact, banking details for invoices).
-- The table itself is created by 0001; this migration seeds the defaults that
-- the application assumes when a key has never been set.
CREATE TABLE IF NOT EXISTS school_settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

INSERT INTO school_settings (key, value) VALUES
  ('school.name', 'School Fees System'),
  ('currency.code', 'KES'),
  ('currency.symbol', 'KSh')
ON CONFLICT(key) DO NOTHING;
