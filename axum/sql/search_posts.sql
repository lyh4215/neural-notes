SELECT id, title, content, created_at, updated_at, user_id, embedding
FROM posts
WHERE user_id = $2
ORDER BY embedding <=> $1
LIMIT 10;
