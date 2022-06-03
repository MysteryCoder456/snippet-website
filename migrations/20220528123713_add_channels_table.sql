-- Add "channels" table
CREATE TABLE channels (
       id     SERIAL PRIMARY KEY,
       name   TEXT
);

-- Remove "receiver_id" from "messages" table
ALTER TABLE messages DROP COLUMN receiver_id;

-- Add one-to-many relationship with channel
ALTER TABLE messages
ADD COLUMN channel_id SERIAL NOT NULL
REFERENCES channels(id) ON DELETE CASCADE;

-- Add one-to-many relationship with sender
ALTER TABLE messages
ADD FOREIGN KEY (sender_id)
REFERENCES users(id);

-- Create many-to-many relationship between user and channel
CREATE TABLE channels_users (
       channel_id    SERIAL REFERENCES channels(id),
       user_id       SERIAL REFERENCES users(id),
       PRIMARY KEY(channel_id, user_id)
);
