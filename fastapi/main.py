from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List
from sentence_transformers import SentenceTransformer
from threading import Lock

# FastAPI 앱 생성
app = FastAPI(title="Local LLM Embedding API")

# Lazy 로딩용 전역 변수
model = None
model_lock = Lock()  # 멀티스레드 환경에서 race condition 방지용

# 요청 모델
class TextRequest(BaseModel):
    text: str

# 응답 모델
class EmbeddingResponse(BaseModel):
    embedding: List[float]

# POST /embed 엔드포인트
@app.post("/embed", response_model=EmbeddingResponse)
async def embed_text(request: TextRequest):
    global model

    # 모델이 아직 로드되지 않았다면 로드 (1회만)
    if model is None:
        with model_lock:
            if model is None:  # 다른 요청에서 이미 로드했을 수도 있으므로 재확인
                try:
                    print("🔵 Loading SentenceTransformer model...")
                    model = SentenceTransformer('all-MiniLM-L6-v2')
                    print("🟢 Model loaded successfully.")
                except Exception as e:
                    print(f"Error loading model: {e}")
                    raise HTTPException(status_code=500, detail="Model loading failed")

    # 임베딩 생성
    try:
        embedding = model.encode(request.text).tolist()
        return EmbeddingResponse(embedding=embedding)
    except Exception as e:
        print(f"Error generating embedding: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/health")
async def health_check():
    return {"status": "ok"}