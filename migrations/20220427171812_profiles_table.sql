-- Add "profiles" table
CREATE TABLE profiles (
       user_id          SERIAL PRIMARY KEY,
       bio              CHAR(200) DEFAULT 'Hi there! I like coding.',
       occupation       CHAR(25) DEFAULT 'Cool Coder',
       default_avatar   BOOLEAN DEFAULT true
);
