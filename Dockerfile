# --- ビルド用ステージ ---
FROM rust:latest as builder

# 必要なシステムライブラリをインストール（SSL通信用）
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
# リリース用にビルド
RUN cargo build --release

# --- 実行用ステージ ---
FROM debian:bookworm-slim
# 実行に必要なランタイムライブラリ（SSL証明書など）をインストール
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# ビルドステージから実行ファイルだけをコピー
COPY --from=builder /app/target/release/mc_log_watcher /app/mc_log_watcher

CMD ["./mc_log_watcher"]
