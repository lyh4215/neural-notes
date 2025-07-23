import os
import httpx
import asyncio
import asyncpg
from dotenv import load_dotenv
import pgvector.asyncpg
from pgvector.asyncpg import Vector, register_vector

load_dotenv()

# Database connection details from environment variables
DB_HOST = os.getenv("DB_HOST", "localhost")
DB_PORT = os.getenv("DB_PORT", "5432")
DB_USER = os.getenv("DB_USER", "user")
DB_PASSWORD = os.getenv("DB_PASSWORD", "password")
DB_NAME = os.getenv("DB_NAME", "neural_notes")

# FastAPI service URL and API key
FASTAPI_URL = os.getenv("FASTAPI_URL", "http://localhost:8001")
FASTAPI_API_KEY = os.getenv("FASTAPI_API_KEY")

async def re_embed_posts():
    if not FASTAPI_API_KEY:
        print("Error: FASTAPI_API_KEY not set in environment variables.")
        return

    conn = None
    try:
        conn = await asyncpg.connect(
            user=DB_USER,
            password=DB_PASSWORD,
            host=DB_HOST,
            port=DB_PORT,
            database=DB_NAME
        )
        await conn.execute("CREATE EXTENSION IF NOT EXISTS vector;")
        await register_vector(conn)
        print("Connected to PostgreSQL database.")

        # Fetch all posts that need re-embedding (e.g., where embedding is not null or content exists)
        # Adjust this query based on your 'posts' table structure and what content you embed
        posts = await conn.fetch("SELECT id, content FROM posts WHERE content IS NOT NULL")
        print(f"Found {len(posts)} posts to re-embed.")

        async with httpx.AsyncClient() as client:
            for post in posts:
                post_id = post['id']
                content = post['content']

                if not content:
                    print(f"Skipping post {post_id}: content is empty.")
                    continue

                try:
                    response = await client.post(
                        f"{FASTAPI_URL}/embed",
                        json={"text": content},
                        headers={"Authorization": f"Bearer {FASTAPI_API_KEY}"},
                        timeout=60.0 # Increased timeout for embedding
                    )
                    response.raise_for_status()
                    embedding_data = response.json()
                    new_embedding = Vector(embedding_data['embedding'])

                    # Update the post with the new embedding
                    await conn.execute(
                        "UPDATE posts SET embedding = $1 WHERE id = $2",
                        new_embedding,
                        post_id
                    )
                    print(f"Successfully re-embedded and updated post {post_id}.")

                except httpx.RequestError as e:
                    print(f"HTTP request failed for post {post_id}: {e}")
                except httpx.HTTPStatusError as e:
                    print(f"HTTP status error for post {post_id}: {e.response.status_code} - {e.response.text}")
                except Exception as e:
                    print(f"An unexpected error occurred for post {post_id}: {e}")

    except Exception as e:
        print(f"Database connection or other error: {e}")
    finally:
        if conn:
            await conn.close()
            print("Disconnected from PostgreSQL database.")

if __name__ == "__main__":
    asyncio.run(re_embed_posts())
