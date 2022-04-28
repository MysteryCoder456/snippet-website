-- Make "language" field in "code_snippets" table NOT NULL
ALTER TABLE code_snippets ALTER COLUMN lang SET NOT NULL;
