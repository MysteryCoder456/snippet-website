-- Merge "users" and "profiles" tables

-- Add profile table's fields to users table
ALTER TABLE users ADD COLUMN bio VARCHAR(200) NOT NULL DEFAULT 'Hi there! I like coding.';
ALTER TABLE users ADD COLUMN occupation VARCHAR(25) NOT NULL DEFAULT 'Cool Coder';
ALTER TABLE users ADD COLUMN avatar_path TEXT;

-- Delete profiles table
DROP TABLE profiles;
