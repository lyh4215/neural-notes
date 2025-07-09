-- Add migration script here
CREATE EXTENSION IF NOT EXISTS vector;

ALTER TABLE posts
    ADD COLUMN IF NOT EXISTS embedding vector(384);

CREATE INDEX IF NOT EXISTS posts_embedding_idx
    ON posts
    USING hnsw (embedding vector_cosine_ops);