# justfile (루트에 위치)


# default help
default:
    just --list

# Rust 백엔드 dev 서버 실행
backend:
    cd axum && REDIS_URL=redis://127.0.0.1:6379 cargo run

# Rust 백엔드 릴리즈 빌드
backend-build:
    cd axum && cargo build --release

# 프론트엔드(dev 서버 실행, Vite 기준)
frontend:
    cd react && npm run dev

# 프론트엔드(production 빌드, Vite/CRA 모두 비슷)
frontend-build:
    cd react && npm run build

# 프론트/백엔드 둘 다 빌드
build-all:
    just backend-build
    just frontend-build

# 도커 컴포즈 전체 실행
up:
    docker compose up --build

# 도커 컴포즈 종료
down:
    docker compose down

# 도커 컴포즈 로그 보기
logs:
    docker compose logs -f

# .env 예시 복사 (옵션)
env:
    cp axum/.env.example axum/.env

# 정리(clean)
clean:
    cd axum && cargo clean
    cd react && rm -rf node_modules dist build
