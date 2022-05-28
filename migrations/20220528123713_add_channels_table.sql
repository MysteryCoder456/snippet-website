-- Add "channels" table
CREATE TABLE channels (
       id   SERIAL PRIMARY KEY,
       name TEXT
);

-- Remove "receiver_id" from "messages" table...
ALTER TABLE messages DROP COLUMN receiver_id;

-- ...and create many-to-one relationship with "channels" table
ALTER TABLE messages
ADD COLUMN channel_id SERIAL NOT NULL
REFERENCES channels(id) ON DELETE CASCADE ON UPDATE CASCADE;

-- Create many-to-many relationship with "channels" and "users" tables
CREATE TABLE channels_users (
       channel_id   SERIAL REFERENCES channels(id),
       user_id      SERIAL REFERENCES users(id),
       PRIMARY KEY(channel_id, user_id)
);
