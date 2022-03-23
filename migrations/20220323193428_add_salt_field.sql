-- Add a salt field to the users table
ALTER TABLE users ADD COLUMN salt CHAR(8);
