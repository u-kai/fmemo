# リファクタリング

## 現状

- main.rsに全てのコードが書かれている

## あるべき姿

- rustはサーバーの起動と、fileのparseのみを行いAPIとしてparseした結果をJSONで返す
- フロントはReactで実装し、APIから取得したJSONを表示する。

## やること

- APIのスキーマを決定して、フロントはparse作業をしなくて済むようにする
- コンポーネント思考でコンポーネントを分割して、使いまわせるようにする
- 可読性を高めて成長可能にする

## 技術スタック

- rust
- react,typescript,tailwindcss
