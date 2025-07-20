# 빌드 스테이지
FROM rust:1.88 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# 실행 스테이지
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/neural_notes_axum /app/server

CMD ["./server"]