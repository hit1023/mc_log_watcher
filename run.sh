#!/bin/bash

case "$1" in
  build)
    echo "📦 Rust版監視ツールをビルドします..."
    docker compose build --no-cache
    ;;
  up)
    echo "🚀 コンテナを起動します..."
    docker compose up -d
    ;;
  down)
    echo "🛑 コンテナを停止・削除します..."
    docker compose down
    ;;
  logs)
    echo "📋 ログを確認します（終了は CTRL+C）..."
    docker compose logs -f
    ;;
  restart)
    echo "🔄 再起動します..."
    docker compose restart
    ;;
  *)
    echo "使い方: $0 {build|up|down|logs|restart}"
    exit 1
esac
