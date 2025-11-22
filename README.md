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
