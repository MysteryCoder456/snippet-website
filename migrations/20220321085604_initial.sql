-- Create the initial database schema
CREATE TABLE code_snippets (
  id            SERIAL PRIMARY KEY,
  author_id     SERIAL NOT NULL,
  title         VARCHAR(100) NOT NULL,
  code          TEXT NOT NULL,
  lang          VARCHAR(20) DEFAULT 'txt',
  created_at    DATE DEFAULT NOW()::date,
  updated_at    DATE DEFAULT NULL
);
