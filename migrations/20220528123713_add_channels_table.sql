-- Add "channels" table
CREATE TABLE channels (
       id            SERIAL PRIMARY KEY,
       name          TEXT,
       members       JSON NOT NULL DEFAULT '[]'
);

-- Remove "receiver_id" from "messages" table and add "channel_id" column
ALTER TABLE messages DROP COLUMN receiver_id;
ALTER TABLE messages ADD COLUMN channel_id SERIAL NOT NULL;
