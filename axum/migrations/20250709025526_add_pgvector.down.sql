-- Revert pgvector extension, posts embedding column, and index

-- 1️⃣ 인덱스 삭제
DROP INDEX IF EXISTS posts_embedding_idx;

-- 2️⃣ 컬럼 삭제
ALTER TABLE posts
    DROP COLUMN IF EXISTS embedding;

-- 3️⃣ pgvector 확장 삭제
DROP EXTENSION IF EXISTS vector;