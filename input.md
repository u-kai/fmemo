# Main Application - COLLAPSIBLE TEST

これはメインアプリケーションです。全体の処理を統括します。
入れ子構造が正しく動作していることを確認します。

```rust
fn main() {
    let app = App::new();
    app.run();
}
```

## User Interface Module

ユーザーインターフェースを管理するモジュールです。

```rust
mod ui {
    pub fn render_window() {
        // ウィンドウを描画
    }
}
```

### Button Component

ボタンコンポーネントの実装です。

```rust
struct Button {
    label: String,
    onclick: fn(),
}
```

#### Button Click Handler

ボタンクリック時の処理です。

```rust
impl Button {
    fn handle_click(&self) {
        (self.onclick)();
    }
}
```

### Menu Component

メニューコンポーネントです。

```rust
struct Menu {
    items: Vec<MenuItem>,
}
```

## Database Module

データベース操作を行うモジュールです。

```rust
mod database {
    pub fn connect() -> Connection {
        // データベースに接続
    }
}
```

### User Repository

ユーザー情報を管理するリポジトリです。

```rust
struct UserRepository {
    connection: Connection,
}
```

#### CRUD Operations

基本的なCRUD操作です。

```rust
impl UserRepository {
    fn create_user(&self, user: User) -> Result<(), Error> {
        // ユーザーを作成
    }
    
    fn find_user(&self, id: i32) -> Option<User> {
        // ユーザーを検索
    }
}
```

##### User Validation

ユーザーデータの検証ロジックです。

```rust
fn validate_user(user: &User) -> bool {
    !user.email.is_empty() && user.age > 0
}
```

## Configuration Module

設定を管理するモジュールです。

```rust
mod config {
    pub fn load_config() -> Config {
        // 設定ファイルを読み込み
    }
}
```