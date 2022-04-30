-- Add "avatar" field to "profiles" table
ALTER TABLE profiles DROP COLUMN default_avatar;
ALTER TABLE profiles ADD COLUMN avatar_path TEXT;
