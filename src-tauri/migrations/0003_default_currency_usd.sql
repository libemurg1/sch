-- Upgrade the original seeded currency without changing customized settings.
UPDATE school_settings
SET value = 'USD'
WHERE key = 'currency.code'
  AND value = 'KES'
  AND EXISTS (
    SELECT 1 FROM school_settings
    WHERE key = 'currency.symbol' AND value = 'KSh'
  );

UPDATE school_settings
SET value = '$'
WHERE key = 'currency.symbol'
  AND value = 'KSh'
  AND EXISTS (
    SELECT 1 FROM school_settings
    WHERE key = 'currency.code' AND value = 'USD'
  );