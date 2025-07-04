# justfile (루트)

# 기본 도움말
default:
    just --list

# =========================
# 🚀 Dev/Build 명령
# =========================

# 전체 dev 서버로 실행 (dev compose)
dev:
    docker image prune -f
    docker compose -f docker-compose.dev.yml up --build

# Rust 백엔드 dev 서버 실행 (로컬)
backend:
    cd axum && REDIS_URL=redis://127.0.0.1:6379 DATABASE_HOST=localhost cargo run

# Rust 백엔드 릴리즈 빌드
backend-build:
    cd axum && cargo build --release

# 프론트엔드(dev)
frontend:
    cd react && npm run dev

# 프론트엔드(production 빌드)
frontend-build:
    cd react && npm run build

# 프론트/백엔드 둘 다 빌드
build-all:
    just backend-build
    just frontend-build

# =========================
# 🐳 Docker Compose
# =========================

# 전체 docker-compose 실행
up:
    docker compose up --build

# docker-compose 종료 및 네트워크 제거
down:
    docker compose down

# docker-compose stop
stop:
    docker compose stop

# docker-compose 로그 보기
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
prune:
    docker system prune -f

# Docker: 미사용 이미지/컨테이너/네트워크/볼륨 모두 삭제 (주의)
prune-all:
    docker system prune -a --volumes -f

# 현재 docker 상태 확인
status:
    docker system df
    docker ps -a

# =========================
# 🛠️ 기타
# =========================

# .env 예시 복사
env:
    cp axum/.env.example axum/.env
