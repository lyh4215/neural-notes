ALTER TABLE posts DROP COLUMN embedding;
ALTER TABLE posts ADD COLUMN embedding vector(384);