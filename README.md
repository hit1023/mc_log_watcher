# mc_log_watcher

Minecraftサーバーのログファイル（`latest.log`）をリアルタイムで監視し、プレイヤーの参加/退出をChatworkに自動通知するRust製の常駐ツールです。座標登録コマンド（`/addloc`）にも対応しています。

## 機能

- プレイヤーが参加した時: `👤 Player <名前> has joined.` をChatworkに投稿
- プレイヤーが退出した時: `🚪 Player <名前> has left.` をChatworkに投稿
- チャット欄で `/addloc <x> <y> <z> <world> <説明>` の形式のコマンドを検知すると:
  - 座標データを外部API（`https://mc.s-quad.com/api/location`）にPOST
  - Chatworkにも座標付きメッセージを投稿
- 複数のMinecraftサーバーのログディレクトリを同時に監視可能（カンマ区切り）
- ログファイルのローテーション（サイズ縮小・再作成）を検知して読み込み位置を自動リセット

## 必要なもの

- Docker / Docker Compose
- Chatwork APIトークン（Chatwork管理画面の「APIトークン」から取得）
- 通知を送りたいChatworkのルームID
- 監視対象のMinecraftサーバー（ログファイル `logs/latest.log` を出力するもの。Paper/Spigot/Vanillaいずれも可）

## セットアップ手順

### 1. リポジトリを取得

```bash
git clone git@github.com:hit1023/mc_log_watcher.git
cd mc_log_watcher
```

### 2. `.env` を作成

`.env.example` をコピーして、実際のChatwork情報を入力してください。

```bash
cp .env.example .env
```

`.env` の中身:

```
CHATWORK_API_TOKEN=your_chatwork_api_token_here
CHATWORK_ROOM_ID=your_chatwork_room_id_here
```

`.env` は `.gitignore` で除外されているため、Gitにコミットされません。**トークンを絶対に公開リポジトリにコミットしないでください。**

### 3. `docker-compose.yml` を自分の環境に合わせて編集

同梱の `docker-compose.yml` は一例です。監視したいMinecraftサーバーのログディレクトリに合わせて `volumes` と環境変数 `LOG_DIRS` を書き換えてください。

```yaml
services:
  rcon-listener:
    build: .
    container_name: rcon-listener
    restart: unless-stopped
    env_file: .env
    environment:
      # コンテナ内のパスをカンマ区切りで指定（複数サーバー監視も可能）
      - LOG_DIRS=/opt/minecraft_server/logs
    volumes:
      # 左側をホスト上の実際のMinecraftサーバーのlogsディレクトリに変更する
      - /path/to/your/minecraft/logs:/opt/minecraft_server/logs:ro
```

複数サーバーを監視する場合の例:

```yaml
    environment:
      - LOG_DIRS=/opt/minecraft_server1/logs,/opt/minecraft_server2/logs
    volumes:
      - /path/to/server1/logs:/opt/minecraft_server1/logs:ro
      - /path/to/server2/logs:/opt/minecraft_server2/logs:ro
```

### 4. ビルド・起動

同梱の `run.sh` で操作できます。

```bash
./run.sh build   # イメージをビルド
./run.sh up      # コンテナを起動
./run.sh logs    # ログを確認（Ctrl+Cで終了）
./run.sh restart # 再起動
./run.sh down    # 停止・削除
```

起動後、ログに以下のような行が出れば監視が始まっています。

```
🟢 Monitoring: "/opt/minecraft_server/logs"
```

### 5. 動作確認

対象のMinecraftサーバーに実際にログイン/ログアウトしてみて、指定したChatworkルームに通知が届くか確認してください。

## 座標登録機能（`/addloc`）について

Minecraftのチャット欄で以下の形式のコマンド（実行するプラグイン等は別途必要）を発言すると、座標がAPIとChatworkの両方に送信されます。

```
/addloc <x> <y> <z> <world名> <説明>
```

送信先のAPIエンドポイント（`https://mc.s-quad.com/api/location`）は `src/main.rs` 内に直書きされています。この機能を使わない場合は無視して構いません（送信に失敗してもエラーは無視され、通知動作全体には影響しません）。自分の環境で座標APIを用意する場合は、`src/main.rs` の `fastapi_url` を書き換えてください。

## トラブルシューティング

- **通知が来ない場合**: `./run.sh logs` でログを確認し、`Monitoring:` の行が出ているか、監視パスが正しいか確認してください。ログファイルのパスやマウント設定のミスが最も多い原因です。
- **Chatworkへの投稿が失敗する場合**: `.env` のトークン・ルームIDが正しいか確認してください（Chatwork API側のエラーは現状ログに出力されないため、トークンを直接 `curl` で試すのが確実です）。
- **ログローテーション検知について**: ファイルサイズが縮小した場合やファイルが再作成された場合、読み込み位置を自動的に0にリセットします。

## ライセンス・注意事項

個人利用を想定した簡易ツールです。Chatwork APIトークンは第三者に公開しないよう管理してください。
