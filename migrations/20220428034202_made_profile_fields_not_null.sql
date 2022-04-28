-- Make "profiles" table fields NOT NULL
ALTER TABLE profiles ALTER COLUMN bio SET NOT NULL;
ALTER TABLE profiles ALTER COLUMN occupation SET NOT NULL;
ALTER TABLE profiles ALTER COLUMN default_avatar SET NOT NULL;
