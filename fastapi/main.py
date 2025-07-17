from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List
from sentence_transformers import SentenceTransformer

# FastAPI 앱 생성
app = FastAPI(title="Local LLM Embedding API")

# SentenceTransformer 모델 로드
# 첫 실행 시 다운로드 후 캐시에 저장되며, 이후 빠르게 로드됨
model = SentenceTransformer('all-MiniLM-L6-v2')  # 384차원, fast, 로컬 CPU/GPU 호환

# 요청 모델
class TextRequest(BaseModel):
    text: str

# 응답 모델
class EmbeddingResponse(BaseModel):
    embedding: List[float]

# POST /embed 엔드포인트
@app.post("/embed", response_model=EmbeddingResponse)
async def embed_text(request: TextRequest):
    try:
        # 임베딩 생성
        embedding = model.encode(request.text).tolist()
        return EmbeddingResponse(embedding=embedding)
    except Exception as e:
        print(f"Error generating embedding: {e}")
        raise HTTPException(status_code=500, detail=str(e))
