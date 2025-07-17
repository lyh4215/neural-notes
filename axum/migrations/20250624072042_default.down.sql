-- Drop triggers
DROP TRIGGER IF EXISTS trigger_update_posts_updated_at ON posts;
DROP TRIGGER IF EXISTS trigger_update_users_updated_at ON users;

-- Drop trigger functions
DROP FUNCTION IF EXISTS update_posts_updated_at();
DROP FUNCTION IF EXISTS update_users_updated_at();

-- Drop tables (in dependency-safe order)
DROP TABLE IF EXISTS comments;
DROP TABLE IF EXISTS posts;
DROP TABLE IF EXISTS users;
