-- Add "messages" table
CREATE TABLE messages (
       id            SERIAL PRIMARY KEY,
       sender_id     SERIAL NOT NULL,
       receiver_id   SERIAL NOT NULL,
       content       TEXT NOT NULL,
       sent_at       TIMESTAMP NOT NULL DEFAULT NOW()
);
