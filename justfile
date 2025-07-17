# justfile (루트)

# 기본 도움말
default:
    @just --list

# =========================
# 🚀 Dev/Build 명령
# =========================

# 전체 dev 서버로 실행 (dev compose)
[group: 'dev']
dev:
    docker image prune -f
    docker compose -f docker-compose.dev.yml up --build

[group: 'dev']
dev-stop:
    docker compose -f docker-compose.dev.yml stop

# Rust 백엔드 dev 서버 실행 (로컬)
[group: 'dev']
backend:
    cd axum && REDIS_URL=redis://127.0.0.1:6379 DATABASE_HOST=localhost cargo run

# Rust 백엔드 릴리즈 빌드
[group: 'prod']
backend-build:
    cd axum && cargo build --release

[group: 'dev']
embed-api:
    cd fastapi && uvicorn main:app --host 0.0.0.0 --port 8001 --reload

# 프론트엔드(dev)
[group: 'dev']
frontend:
    cd react && npm run dev

# 프론트엔드(production 빌드)
[group: 'prod']
frontend-build:
    cd react && npm run build

# 프론트/백엔드 둘 다 빌드 (Docker 이미지로)
[group: 'build']
build-all:
    docker build -t neural-notes-axum:latest ./axum
    docker build -t neural-notes-fastapi:latest ./fastapi
    docker build -t neural-notes-react:latest ./react

#kind cluster 생성
[group: 'k8s']
kind-create:
    kind create cluster --config kind.yaml

# kind 클러스터에 이미지 등록
[group: 'k8s']
kind-load-all:
    kind load docker-image neural-notes-axum:latest
    kind load docker-image neural-notes-fastapi:latest
    kind load docker-image neural-notes-react:latest

# 빌드 + 로드까지 한 번에 하고 싶을 때
[group: 'k8s']
build-and-load:
    just build-all
    just kind-load-all

[group: 'k8s']
deploy:
    kubectl apply -f k8s/redis.yaml
    kubectl apply -f k8s/postgres.yaml
    kubectl apply -f k8s/axum.yaml
    kubectl apply -f k8s/fastapi.yaml
    kubectl apply -f k8s/react.yaml

[group: 'k8s']
undeploy:
    -kubectl delete -f k8s/react.yaml
    -kubectl delete -f k8s/fastapi.yaml
    -kubectl delete -f k8s/axum.yaml
    -kubectl delete -f k8s/postgres.yaml
    -kubectl delete -f k8s/redis.yaml

# =========================
# 🐳 Docker Compose
# =========================

# 전체 docker-compose 실행
[group: 'docker']
up:
    docker compose up --build

# docker-compose 종료 및 네트워크 제거
[group: 'docker']
down:
    docker compose down

# docker-compose stop
[group: 'docker']
stop:
    docker compose stop

# docker-compose 로그 보기
[group: 'docker']
logs:
    docker compose logs -f

# =========================
# 🧹 정리 및 용량 관리
# =========================

# Rust/Node 빌드 캐시 및 node_modules 삭제
clean:
    cd axum && cargo clean
    cd react && rm -rf node_modules dist build

# Docker: dangling 이미지, 캐시, 미사용 네트워크 삭제 (안전)
[group: 'docker']
prune:
    docker system prune -f

# Docker: 미사용 이미지/컨테이너/네트워크/볼륨 모두 삭제 (주의)
[group: 'docker']
prune-all:
    docker system prune -a --volumes -f

# 현재 docker 상태 확인
[group: 'docker']
status:
    docker system df
    docker ps -a

# =========================
# 🛠️ 기타
# =========================

# .env 예시 복사
[group: 'utils']
env:
    cp axum/.env.example axum/.env
    cp react/.env.example react/.env
    cp .env.example .env
