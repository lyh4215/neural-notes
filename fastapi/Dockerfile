FROM python:3.10-slim

RUN apt-get update && apt-get install -y build-essential && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# 작업 디렉토리 설정
WORKDIR /app

# 종속성 복사 및 설치
COPY requirements.txt ./
# torch 먼저 설치 (CPU 버전)
RUN pip install --no-cache-dir -r requirements.txt && \
    rm -rf /root/.cache /root/.huggingface /usr/local/lib/python3.10/site-packages/__pycache__


# FastAPI 소스 복사
COPY . .

# uvicorn 실행
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8001"]
