-- Change "profiles" table CHAR fields to VARCHAR
ALTER TABLE profiles ALTER COLUMN bio TYPE VARCHAR(200);
ALTER TABLE profiles ALTER COLUMN occupation TYPE VARCHAR(25);
