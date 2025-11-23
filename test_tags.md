# メイン関数

<desc>アプリケーションのエントリーポイントです</desc>
<path>src/main.rs:10</path>

```rust
fn main() {
    println!("Hello World");
}
```

## 設定読み込み

<desc>設定ファイルを読み込んで初期化する関数</desc>
<path>src/config.rs:25</path>

```rust
fn load_config() -> Config {
    Config::from_file("config.toml")
}
```

### 設定解析

<desc>TOML形式の設定を解析</desc>
<path>src/config.rs:45</path>

## データベース接続

<desc>PostgreSQLデータベースへの接続を確立</desc>
<path>src/db/connection.rs:15</path>

### クエリ実行

<desc>SQLクエリを実行して結果を返す</desc>
<path>src/db/query.rs:30</path>

#### SELECT文

<desc>データを選択するクエリ</desc>
<path>src/db/query.rs:45</path>

##### ジョイン処理

<desc>複数テーブルを結合する処理</desc>
<path>src/db/query.rs:78</path>