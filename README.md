# Fmemo - Function Memo Tool

階層構造でコードの理解を深めるMarkdownベースの可視化ツール

## 概要

コードの関数やモジュールの関係を理解しながらMarkdownでメモを作成し、リアルタイムでインタラクティブな階層表示を生成するツールです。

## 主な機能

- 📝 **Markdown形式でメモ作成** - 階層構造（#, ##, ###）でコードの関係を記述
- 🖼️ **リアルタイム可視化** - ファイル変更を自動検知してWebUI更新
- 🔍 **2つの表示モード**：
  - **Memoモード**: 詳細な説明とコードブロックを含む階層表示
  - **Flowモード**: タイトルと関係性に特化したフロー図
- 🎯 **カスタムタグ対応**：
  - `<desc>説明</desc>` - Flow表示用の説明文
  - `<path>src/main.rs:10</path>` - コードの場所
- ⚡ **インタラクティブ機能**：
  - ズーム/パン操作
  - 展開/折りたたみ
  - Flowノードクリックで対象箇所にジャンプ
- 🎨 **レスポンシブUI** - 水平/垂直レイアウト対応

## 技術構成

### Backend (Rust)
- **warp** - Webサーバー・WebSocket
- **notify** - ファイル監視
- **clap** - CLI引数解析

### Frontend (React)
- **React 18 + TypeScript**
- **Tailwind CSS**
- **Atomic Design** パターン
- **Vite** - 開発サーバー

## 使用方法

### 開発サーバー起動
```bash
# Backend (Rust)
cargo run -- -f input.md -p 3030

# Frontend (React)
cd frontend && npm run dev
```

### Markdownの記述例

```markdown
# メイン処理

<desc>アプリケーションのエントリーポイント</desc>
<path>src/main.rs:10</path>

## 設定読み込み

<desc>設定ファイルを解析して初期化</desc>
<path>src/config.rs:25</path>

### TOML解析

<desc>設定ファイルの構文解析処理</desc>
<path>src/config.rs:45</path>
```

## プロジェクト構成

```
fmemo/
├── src/main.rs              # Rustバックエンド
├── frontend/                # Reactフロントエンド
│   ├── src/
│   │   ├── components/
│   │   │   ├── atoms/       # 基本コンポーネント
│   │   │   └── molecules/   # 組み合わせコンポーネント
│   │   └── types/           # TypeScript型定義
├── input.md                 # サンプル入力ファイル
└── README.md               # このファイル
```
