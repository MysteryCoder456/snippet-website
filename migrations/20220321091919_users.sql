-- Add "users" table schema
CREATE TABLE users (
    id          SERIAL PRIMARY KEY,
    username    VARCHAR(100) NOT NULL,
    email       VARCHAR(100) NOT NULL,
    passwd      TEXT NOT NULL,
    created_at  DATE DEFAULT NOW()::date
);
