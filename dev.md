# Development Setup

開発時にホットリロードでフロントエンドを動かすセットアップです。

## セットアップ手順

### 1. Rustバックエンド（API）サーバーを起動

```bash
# ルートディレクトリで
cargo run -- --dev -r .
```

このコマンドは以下を実行します：
- ポート3030でAPIサーバーを起動
- .fmemoファイルを監視してリアルタイム更新
- フロントエンドは含まず、API専用モード

### 2. Reactフロントエンドの開発サーバーを起動

別のターミナルで：

```bash
cd frontend
npm install  # 初回のみ
npm run dev
```

これにより：
- ポート5173（Viteのデフォルト）でReactアプリが起動
- `/api/*`と`/ws`のリクエストは自動的にport 3030のRustサーバーにプロキシされます
- ホットリロードが有効

### 3. アクセス

- フロントエンド: http://localhost:5173 （ホットリロード付き）
- API直接: http://localhost:3030/api/root

## プロキシ設定

`frontend/vite.config.ts`で以下が設定されています：

```typescript
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:3030',
      changeOrigin: true
    },
    '/ws': {
      target: 'ws://localhost:3030', 
      ws: true
    }
  }
}
```

## テスト用ファイル

プロジェクトルートに以下のテストファイルがあります：
- `test.fmemo` - シンプルなテスト関数
- `example.fmemo` - アプリケーション例

## コマンド一覧

```bash
# 開発モード（API + ホットリロード）
cargo run -- --dev

# 本番モード（API + 静的ファイル配信）
cargo run -- -f ./frontend/dist

# API専用モード
cargo run -- --api-only

# ヘルプ
cargo run -- --help
```