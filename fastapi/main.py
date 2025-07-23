import os
from fastapi import FastAPI, Depends, HTTPException, status
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel
from typing import List
from sentence_transformers import SentenceTransformer
from threading import Lock
from dotenv import load_dotenv
import secrets

load_dotenv()

security = HTTPBearer()

async def get_api_key(credentials: HTTPAuthorizationCredentials = Depends(security)):
    token = credentials.credentials
    if secrets.compare_digest(token, os.getenv("FASTAPI_API_KEY")):
        return token
    raise HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Invalid authentication credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )

# FastAPI 앱 생성
app = FastAPI(title="Local LLM Embedding API")

# Lazy 로딩용 전역 변수
model = None
model_lock = Lock()  # 멀티스레드 환경에서 race condition 방지용

def get_model():
    global model
    if model is None:
        with model_lock:
            if model is None:
                try:
                    print("🔵 Loading SentenceTransformer model...")
                    model = SentenceTransformer('distiluse-base-multilingual-cased-v2')
                    print("🟢 Model loaded successfully.")
                except Exception as e:
                    print(f"Error loading model: {e}")
                    raise HTTPException(status_code=500, detail="Model loading failed")
    return model

# 요청 모델
class TextRequest(BaseModel):
    text: str

class QueryRequest(BaseModel):
    query: str

# 응답 모델
class EmbeddingResponse(BaseModel):
    embedding: List[float]

# POST /embed 엔드포인트
@app.post("/embed", response_model=EmbeddingResponse, dependencies=[Depends(get_api_key)])
async def embed_text(request: TextRequest, model: SentenceTransformer = Depends(get_model)):
    try:
        embedding = model.encode(request.text).tolist()
        return EmbeddingResponse(embedding=embedding)
    except Exception as e:
        print(f"Error generating embedding: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/query-embedding", response_model=EmbeddingResponse, dependencies=[Depends(get_api_key)])
async def query_embedding(request: QueryRequest, model: SentenceTransformer = Depends(get_model)):
    print(f"Received search query in FastAPI: {request.query}")
    try:
        embedding = model.encode(request.query).tolist()
        return EmbeddingResponse(embedding=embedding)
    except Exception as e:
        print(f"Error generating embedding: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/health")
async def health_check():
    return {"status": "ok"}