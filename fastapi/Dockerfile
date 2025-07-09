FROM python:3.10-slim

# 시각화, 디버깅 편의를 위한 기본 패키지
RUN apt-get update && apt-get install -y build-essential

# 작업 디렉토리 설정
WORKDIR /app

# 종속성 복사 및 설치
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# FastAPI 소스 복사
COPY . .

# uvicorn 실행
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8001", "--reload"]
