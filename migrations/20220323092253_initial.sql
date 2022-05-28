-- Create the initial database schema

CREATE TABLE users (
    id          SERIAL PRIMARY KEY,
    username    VARCHAR(100) NOT NULL,
    email       VARCHAR(100) NOT NULL,
    passwd      TEXT NOT NULL,
    created_at  TIMESTAMP DEFAULT NOW()
);

CREATE TABLE code_snippets (
    id            SERIAL PRIMARY KEY,
    author_id     SERIAL NOT NULL,
    title         VARCHAR(100) NOT NULL,
    code          TEXT NOT NULL,
    lang          VARCHAR(20) DEFAULT 'txt',
    created_at    TIMESTAMP DEFAULT NOW(),
    updated_at    TIMESTAMP DEFAULT NULL
);
