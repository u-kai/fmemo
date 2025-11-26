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

## Installation

### Building from Source

```bash
# Build single binary with embedded frontend
make build

# Install to /usr/local/bin (requires sudo)
sudo make install

# Now you can run fmemo from anywhere
fmemo -r ~/my-memos -p 3030
```

### Uninstall

```bash
sudo make uninstall
```

## Usage

### Basic Usage

```bash
# Start server with default settings (current directory, port 3030)
fmemo

# Specify custom directory and port
fmemo -r ~/my-memos -p 8080

# Development mode (API only, for use with separate frontend dev server)
fmemo --dev

# API only mode (no frontend hosting)
fmemo --api-only
```

### Command Line Options

```
Options:
  -r, --root <ROOT_DIR>          Root directory to serve .fmemo files from [default: .]
  -p, --port <PORT>              Port to serve on [default: 3030]
  -f, --frontend <FRONTEND_DIR>  Frontend dist directory (optional)
      --api-only                 Run API server only, without frontend hosting
      --dev                      Development mode - serve API only
  -h, --help                     Print help
  -V, --version                  Print version
```

### Makefile Targets

```bash
# Build single binary with embedded frontend
make build

# Build frontend and package into binary
make package

# Install to /usr/local/bin (default)
sudo make install

# Install to custom directory
sudo make install INSTALL_DIR=/opt/bin

# Uninstall
sudo make uninstall

# Run the server
make run

# Run tests and verify endpoints
make verify

# Clean build artifacts
make clean
```

## Development

### Prerequisites

- Rust (latest stable)
- Node.js and npm (for building frontend)

### Build Steps

1. **Build frontend** (first time only, requires Node.js):
   ```bash
   cd frontend && npm ci && npm run build
   ```

2. **Build Rust binary with embedded frontend**:
   ```bash
   cargo build --release --features embed_frontend
   ```

3. **Run**:
   ```bash
   ./target/release/fmemo -r . -p 3030
   ```

### Notes

- **Single Binary**: After building with `embed_frontend` feature, the binary is completely standalone and doesn't require Node.js or frontend files at runtime
- **Runtime Modes**:
  - With `embed_frontend` feature: Serves embedded frontend from binary
  - Without feature: Auto-detects `frontend/dist` directory or runs API-only
  - With `--frontend` flag: Serves frontend from specified directory
- **Development**: Use `--dev` flag to run API-only server while running frontend separately with `npm run dev`

## API Endpoints

- `GET /api/root` - Get directory tree of .fmemo files
- `GET /api/files/{filename}` - Get file content
- `GET /api/file/{filename}` - Get file content (frontend compatible)
- `WebSocket /ws` - Real-time file system updates
