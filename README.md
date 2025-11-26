# Function Memo Tool

- コードを読む際にメモとして可視化するツール
- 関数の呼び出し連鎖をMarkdown形式で記述していきながらそのメモが可視化される
- 例えば以下のようなコードがある場合

````md
# RootFunction

- RootFunctionの実装は以下
  - helper_functionを呼び出している

```rs
fn root_function() {
    helper_function();
    helper_function2();
}
```

## helper_function

- helper_functionの実装は以下

```rs
fn helper_function() {
    println!("Hello, world!");
}
```

## helper_function2

- helper_function2はよくわからない。。一旦スキップする
````

->

# (以下のようなhtmlを生成する)

|======================================
|
|# RootFunction
|- RootFunctionの実装は以下
| - helper_functionを呼び出している
|
|`rs
|fn root_function() {
|    helper_function();
|    helper_function2();
|}
|`
|=============================|
|## helper_function |
|- helper_functionの実装は以下|
| |
|`rs                          |
|fn helper_function() {       |
|    println!("Hello, world!")|;
|}                            |
|` |
| |
|======================= |
|## helper_function2
|- helper_function2はよくわからない。。一旦スキップする
|
|
|======================================

## Makefile Usage (Single Binary)

- Build frontend, embed into Rust binary, run and verify
  - `make package` — build frontend (`frontend/dist`) and Rust release binary with `embed_frontend`
  - `make run ROOT=. PORT=3030` — run the binary serving embedded SPA + API/WS
  - `make verify ROOT=. PORT=3030` — start in background, check `/api/root` and `/` (index.html), then stop

- Notes
  - First time only: Node.js required to build frontend assets
  - After packaging, runtime does not require Node; `./target/release/fmemo -r <root>` works standalone
  - You can also pass `--frontend ./frontend/dist` to serve from disk without embedding
