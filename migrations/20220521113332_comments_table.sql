-- Add "comments" table
CREATE TABLE comments (
       id               SERIAL PRIMARY KEY,
       code_snippet_id  SERIAL NOT NULL,
       author_id        SERIAL NOT NULL,
       content          TEXT NOT NULL
);
