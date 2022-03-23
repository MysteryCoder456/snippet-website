-- Make "salt" field NOT NULL
ALTER TABLE users ALTER COLUMN salt SET NOT NULL;
