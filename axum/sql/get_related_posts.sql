SELECT
p1.id AS source,
p2.id AS target,
(p1.embedding <-> p2.embedding) AS distance
FROM posts p1
JOIN LATERAL (
    SELECT id, embedding
    FROM posts p2
    WHERE p2.user_id = p1.user_id
    AND p2.id != p1.id
    AND p2.embedding IS NOT NULL
    AND p1.embedding <-> p2.embedding < $2
    ORDER BY p2.embedding <-> p1.embedding
    LIMIT 5
) p2 ON TRUE
WHERE p1.embedding IS NOT NULL
AND p1.user_id = $1;