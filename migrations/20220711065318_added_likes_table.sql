-- Add "likes" table
CREATE TABLE likes (
    liker_id    SERIAL REFERENCES users(id),
    snippet_id  SERIAL REFERENCES code_snippets(id),
    PRIMARY KEY(liker_id, snippet_id)
);
