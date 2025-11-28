use crate::parser::parse_memo;
use crate::schema::{DirectoryTree, FileContent};
use futures_util::{SinkExt, StreamExt};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc::UnboundedSender;
use warp::Filter;

/// WebSocket client manager
pub type WebSocketClients = Arc<Mutex<Vec<UnboundedSender<warp::ws::Message>>>>;

/// File change notification data
#[derive(Debug, Clone)]
pub struct FileChangeNotification {
    pub file_path: PathBuf,
    pub memos: Vec<crate::schema::Memo>,
}

/// Scan directory for .fmemo files and build directory tree
pub fn scan_directory<P: AsRef<Path>>(root_path: P) -> std::io::Result<DirectoryTree> {
    let root_path = root_path.as_ref();
    let mut files = Vec::new();
    let mut subdirectories = Vec::new();

    if !root_path.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Path is not a directory",
        ));
    }

    for entry in fs::read_dir(root_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "fmemo" || ext == "md" {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        files.push(file_name.to_string());
                    }
                }
            }
        } else if path.is_dir() {
            // Skip hidden directories
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if !dir_name.starts_with('.') {
                    let subdir_tree = scan_directory(&path)?;
                    // Only include subdirectories that contain .fmemo files (recursively)
                    if has_fmemo_files(&subdir_tree) {
                        subdirectories.push(subdir_tree);
                    }
                }
            }
        }
    }

    Ok(DirectoryTree {
        path: root_path.to_string_lossy().to_string(),
        files,
        subdirectories,
    })
}

/// Check if directory tree contains any .fmemo files (recursively)
fn has_fmemo_files(tree: &DirectoryTree) -> bool {
    !tree.files.is_empty() || tree.subdirectories.iter().any(has_fmemo_files)
}

/// Read and parse a .fmemo file
pub fn read_fmemo_file<P: AsRef<Path>>(file_path: P) -> std::io::Result<FileContent> {
    let file_path = file_path.as_ref();
    
    // Verify it's a .fmemo or .md file
    let ext = file_path.extension().and_then(|s| s.to_str());
    if ext != Some("fmemo") && ext != Some("md") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "File must have .fmemo or .md extension",
        ));
    }

    let content = fs::read_to_string(file_path)?;
    let memos = parse_memo(&content);
    
    // Get last modified time
    let last_modified = file_path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    Ok(FileContent {
        memos,
        last_modified,
    })
}

/// Create static file serving routes for React frontend
pub fn create_static_routes(
    dist_dir: PathBuf,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // Serve static assets (CSS, JS, etc.)
    let static_files = warp::path("assets")
        .and(warp::fs::dir(dist_dir.join("assets")));
    
    // Serve favicon and other root files
    let favicon = warp::path("favicon.ico")
        .and(warp::fs::file(dist_dir.join("favicon.ico")));
    
    let vite_svg = warp::path("vite.svg")
        .and(warp::fs::file(dist_dir.join("vite.svg")));
    
    // Catch all route for SPA - serve index.html for all non-API, non-WS routes
    let spa_routes = warp::get()
        .and(warp::path::full())
        .and(warp::fs::file(dist_dir.join("index.html")))
        .map(|_path: warp::path::FullPath, file| file);
    
    static_files
        .or(favicon)
        .or(vite_svg)
        .or(spa_routes)
}

/// Create API routes
pub fn create_api_routes(
    root_dir: PathBuf,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let root_route = {
        let root_dir = root_dir.clone();
        warp::path!("api" / "root")
            .and(warp::get())
            .map(move || {
                match scan_directory(&root_dir) {
                    Ok(tree) => {
                        // Return full hierarchical structure
                        warp::reply::with_status(
                            warp::reply::json(&tree),
                            warp::http::StatusCode::OK,
                        )
                    }
                    Err(_) => {
                        warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": "Failed to scan directory"})),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    }
                }
            })
    };

    let files_route = {
        let root_dir = root_dir.clone();
        warp::path!("api" / "files" / String)
            .and(warp::get())
            .map(move |filename: String| {
                let file_path = root_dir.join(&filename);
                
                match read_fmemo_file(&file_path) {
                    Ok(content) => {
                        warp::reply::with_status(
                            warp::reply::json(&content),
                            warp::http::StatusCode::OK,
                        )
                    }
                    Err(e) => {
                        let error_msg = match e.kind() {
                            std::io::ErrorKind::NotFound => "File not found",
                            std::io::ErrorKind::InvalidInput => "Invalid file type (must be .fmemo or .md)",
                            _ => "Failed to read file",
                        };
                        warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": error_msg})),
                            warp::http::StatusCode::NOT_FOUND,
                        )
                    }
                }
            })
    };

    // Add compatibility route for frontend API client
    // Support nested paths for files (e.g., sub/dir/file.fmemo)
    let file_route = {
        let root_dir = root_dir.clone();
        warp::path("api")
            .and(warp::path("file"))
            .and(warp::path::tail())
            .and(warp::get())
            .map(move |tail: warp::path::Tail| {
                // Simple URL decode for %2F -> /
                let filename = tail.as_str().replace("%2F", "/").replace("%2f", "/");
                let file_path = root_dir.join(&filename);

                match read_fmemo_file(&file_path) {
                    Ok(content) => {
                        // Transform to frontend expected format
                        let response = serde_json::json!({
                            "path": filename,
                            "content": format!("# {}\n\nParsed from fmemo file", filename),
                            "memos": content.memos
                        });
                        warp::reply::with_status(
                            warp::reply::json(&response),
                            warp::http::StatusCode::OK,
                        )
                    }
                    Err(e) => {
                        let error_msg = match e.kind() {
                            std::io::ErrorKind::NotFound => "File not found",
                            std::io::ErrorKind::InvalidInput => "Invalid file type (must be .fmemo or .md)",
                            _ => "Failed to read file",
                        };
                        warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": error_msg})),
                            warp::http::StatusCode::NOT_FOUND,
                        )
                    }
                }
            })
    };

    // Add CORS headers for API routes
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    root_route
        .or(files_route)
        .or(file_route)
        .with(cors)
}

/// Create full server routes (API + WebSocket + optionally static files)
pub fn create_full_routes(
    root_dir: PathBuf,
    dist_dir: PathBuf,
    clients: WebSocketClients,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let api_routes = create_api_routes(root_dir);
    let ws_route = create_websocket_route(clients);
    let static_routes = create_static_routes(dist_dir);
    
    api_routes.or(ws_route).or(static_routes)
}

/// Create API-only routes (API + WebSocket)
pub fn create_api_only_routes(
    root_dir: PathBuf,
    clients: WebSocketClients,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let api_routes = create_api_routes(root_dir);
    let ws_route = create_websocket_route(clients);
    
    api_routes.or(ws_route)
}

// Embedded static file serving (feature-gated)
#[cfg(feature = "embed_frontend")]
mod embedded {
    use super::*;
    use mime_guess::from_path;
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "frontend/dist/"]
    struct Assets;

    fn respond(path: &str) -> Option<warp::reply::Response> {
        let asset = Assets::get(path)?;
        let body = warp::hyper::Body::from(asset.data.into_owned());
        let mime = from_path(path).first_or_octet_stream();
        let mut resp = warp::http::Response::new(body);
        resp.headers_mut().insert(
            warp::http::header::CONTENT_TYPE,
            warp::http::HeaderValue::from_str(mime.as_ref()).unwrap_or(warp::http::HeaderValue::from_static("application/octet-stream")),
        );
        Some(resp)
    }

    pub fn create_embedded_static_routes(
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        // /assets/*
        let assets = warp::path("assets")
            .and(warp::path::tail())
            .and(warp::get())
            .map(|tail: warp::path::Tail| {
                let path = format!("assets/{}", tail.as_str());
                respond(&path).unwrap_or_else(|| warp::reply::Response::new(warp::hyper::Body::empty()))
            });

        // Root-level files like index.html, favicon.ico, vite.svg
        let root_files = warp::get().and(warp::path::param::<String>()).and_then(|name: String| async move {
            let known = ["index.html", "favicon.ico", "vite.svg"]; // fast path
            if known.contains(&name.as_str()) {
                if let Some(resp) = respond(&name) { return Ok(resp); }
            }
            Err(warp::reject::not_found())
        });

        // SPA fallback: serve index.html for all non-API, non-WS GET routes
        let spa = warp::get()
            .and(warp::path::full())
            .and_then(|_path: warp::path::FullPath| async move {
                if let Some(resp) = respond("index.html") {
                    Ok::<_, warp::reject::Rejection>(resp)
                } else {
                    Err(warp::reject::not_found())
                }
            });

        assets.or(root_files).or(spa)
    }
}

#[cfg(feature = "embed_frontend")]
pub fn create_full_routes_embedded(
    root_dir: PathBuf,
    clients: WebSocketClients,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let api_routes = create_api_routes(root_dir);
    let ws_route = create_websocket_route(clients);
    let static_routes = embedded::create_embedded_static_routes();
    api_routes.or(ws_route).or(static_routes)
}

/// Create WebSocket route for real-time updates
pub fn create_websocket_route(
    clients: WebSocketClients,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let clients = Arc::clone(&clients);
            ws.on_upgrade(move |websocket| async move {
                handle_websocket_connection(websocket, clients).await;
            })
        })
}

/// Handle individual WebSocket connection
async fn handle_websocket_connection(
    websocket: warp::ws::WebSocket,
    clients: WebSocketClients,
) {
    let (mut ws_tx, mut ws_rx) = websocket.split();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    clients.lock().unwrap().push(tx);

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            if result.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

/// Broadcast message to all WebSocket clients
pub fn broadcast_to_clients(clients: &WebSocketClients, message: serde_json::Value) {
    let clients_lock = clients.lock().unwrap();
    let message_text = message.to_string();
    
    clients_lock.iter().for_each(|client| {
        let _ = client.send(warp::ws::Message::text(message_text.clone()));
    });
}

/// Start file watcher for a specific file
pub fn start_file_watcher<P: AsRef<Path>>(
    file_path: P,
    clients: WebSocketClients,
) -> std::io::Result<()> {
    let file_path = file_path.as_ref().to_path_buf();
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    watcher.watch(&file_path, RecursiveMode::NonRecursive)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Spawn background thread to handle file changes
    thread::spawn(move || {
        // Keep watcher alive
        let _watcher = watcher;
        
        loop {
            match rx.recv() {
                Ok(Ok(_event)) => {
                    if let Ok(content) = fs::read_to_string(&file_path) {
                        let memos = parse_memo(&content);
                        
                        let update_msg = serde_json::json!({
                            "type": "file_updated",
                            "file_path": file_path.to_string_lossy(),
                            "memos": memos
                        });
                        
                        broadcast_to_clients(&clients, update_msg);
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("File watch event error: {:?}", e);
                }
                Err(e) => {
                    eprintln!("File watch channel error: {:?}", e);
                    break;
                }
            }
        }
    });

    Ok(())
}

/// Start directory watcher for .fmemo files
pub fn start_directory_watcher<P: AsRef<Path>>(
    root_path: P,
    clients: WebSocketClients,
) -> std::io::Result<()> {
    let root_path = root_path.as_ref().to_path_buf();
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    watcher.watch(&root_path, RecursiveMode::Recursive)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    thread::spawn(move || {
        let _watcher = watcher;
        let mut last_processed: std::collections::HashMap<std::path::PathBuf, std::time::SystemTime> = std::collections::HashMap::new();
        
        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    use std::collections::HashSet;
                    use notify::EventKind;
                    
                    // Only process actual file content changes
                    if !matches!(event.kind, 
                        EventKind::Modify(notify::event::ModifyKind::Data(_)) | 
                        EventKind::Create(_)
                    ) {
                        continue;
                    }
                    
                    let now = std::time::SystemTime::now();
                    let mut processed_files = HashSet::new();
                    
                    // Check if any changed file is a .fmemo or .md file
                    for path in &event.paths {
                        let ext = path.extension().and_then(|s| s.to_str());
                        if (ext == Some("fmemo") || ext == Some("md")) && 
                           processed_files.insert(path.clone()) {
                            
                            // Check if we processed this file recently (within 2 seconds)
                            if let Some(last_time) = last_processed.get(path) {
                                if let Ok(duration) = now.duration_since(*last_time) {
                                    if duration.as_secs() < 2 {
                                        println!("Skipping recent file change: {}", path.display());
                                        continue;
                                    }
                                }
                            }
                            
                            // Update last processed time
                            last_processed.insert(path.clone(), now);
                            
                            // Send individual file update message
                            if let Ok(content) = fs::read_to_string(path) {
                                let memos = parse_memo(&content);
                                
                                let file_update_msg = serde_json::json!({
                                    "type": "file_updated",
                                    "file_path": path.to_string_lossy(),
                                    "path": path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                                    "memos": memos
                                });
                                
                                broadcast_to_clients(&clients, file_update_msg);
                                println!("Sent file update for: {}", path.display());
                            }
                        }
                    }

                    // If structure changed (create/remove/rename), broadcast directory update
                    if matches!(event.kind,
                        EventKind::Create(_) |
                        EventKind::Remove(_) |
                        EventKind::Modify(notify::event::ModifyKind::Name(_))
                    ) {
                        if let Ok(tree) = scan_directory(&root_path) {
                            // Transform to frontend expected format
                            let response = serde_json::json!({
                                "files": tree.files,
                                "directories": tree.subdirectories.iter().map(|subdir| {
                                    std::path::Path::new(&subdir.path)
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(&subdir.path)
                                }).collect::<Vec<_>>()
                            });

                            let dir_msg = serde_json::json!({
                                "type": "directory_updated",
                                "tree": response
                            });
                            broadcast_to_clients(&clients, dir_msg);
                            println!("Sent directory update for root: {}", root_path.display());
                        }
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Directory watch event error: {:?}", e);
                }
                Err(e) => {
                    eprintln!("Directory watch channel error: {:?}", e);
                    break;
                }
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_fmemo_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(format!("{}.fmemo", name));
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = scan_directory(temp_dir.path()).unwrap();
        
        assert_eq!(result.files.len(), 0);
        assert_eq!(result.subdirectories.len(), 0);
    }

    #[test]
    fn test_scan_directory_with_fmemo_files() {
        let temp_dir = TempDir::new().unwrap();
        
        create_test_fmemo_file(temp_dir.path(), "test1", "# Test 1\nContent 1");
        create_test_fmemo_file(temp_dir.path(), "test2", "# Test 2\nContent 2");
        
        // Create non-fmemo file (should be ignored)
        fs::write(temp_dir.path().join("ignored.md"), "# Ignored").unwrap();
        
        let result = scan_directory(temp_dir.path()).unwrap();
        
        assert_eq!(result.files.len(), 2);
        assert!(result.files.contains(&"test1.fmemo".to_string()));
        assert!(result.files.contains(&"test2.fmemo".to_string()));
        assert_eq!(result.subdirectories.len(), 0);
    }

    #[test]
    fn test_scan_directory_with_subdirectories() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create files in root
        create_test_fmemo_file(temp_dir.path(), "root", "# Root");
        
        // Create subdirectory with fmemo files
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        create_test_fmemo_file(&sub_dir, "sub1", "# Sub 1");
        create_test_fmemo_file(&sub_dir, "sub2", "# Sub 2");
        
        // Create empty subdirectory (should be excluded)
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir(&empty_dir).unwrap();
        
        let result = scan_directory(temp_dir.path()).unwrap();
        
        assert_eq!(result.files.len(), 1);
        assert_eq!(result.subdirectories.len(), 1);
        
        let subdir_tree = &result.subdirectories[0];
        assert_eq!(subdir_tree.files.len(), 2);
        assert!(subdir_tree.files.contains(&"sub1.fmemo".to_string()));
        assert!(subdir_tree.files.contains(&"sub2.fmemo".to_string()));
    }

    #[test]
    fn test_scan_directory_skips_hidden_dirs() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create hidden directory with fmemo file
        let hidden_dir = temp_dir.path().join(".hidden");
        fs::create_dir(&hidden_dir).unwrap();
        create_test_fmemo_file(&hidden_dir, "hidden", "# Hidden");
        
        // Create normal directory with fmemo file
        let normal_dir = temp_dir.path().join("normal");
        fs::create_dir(&normal_dir).unwrap();
        create_test_fmemo_file(&normal_dir, "normal", "# Normal");
        
        let result = scan_directory(temp_dir.path()).unwrap();
        
        assert_eq!(result.subdirectories.len(), 1);
        assert_eq!(result.subdirectories[0].files[0], "normal.fmemo");
    }

    #[test]
    fn test_read_fmemo_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_content = r#"
# Test Function
<desc>A test function</desc>

This is some content.

```rust
fn test() {
    println!("test");
}
```

## Sub Section
More content here.
"#;
        
        let file_path = create_test_fmemo_file(temp_dir.path(), "test", file_content);
        
        let result = read_fmemo_file(&file_path).unwrap();
        
        assert_eq!(result.memos.len(), 1);
        assert_eq!(result.memos[0].title(), "Test Function");
        assert_eq!(result.memos[0].description(), &Some("A test function".to_string()));
        assert_eq!(result.memos[0].code_blocks().len(), 1);
        assert_eq!(result.memos[0].children().len(), 1);
        assert_eq!(result.memos[0].children()[0].title(), "Sub Section");
        assert!(result.last_modified.is_some());
    }

    #[test]
    fn test_read_non_fmemo_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        fs::write(&file_path, "# Test").unwrap();
        
        let result = read_fmemo_file(&file_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.fmemo");
        
        let result = read_fmemo_file(&file_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_has_fmemo_files() {
        // Tree with files
        let tree_with_files = DirectoryTree {
            path: "/test".to_string(),
            files: vec!["test.fmemo".to_string()],
            subdirectories: vec![],
        };
        assert!(has_fmemo_files(&tree_with_files));

        // Tree without files but with subdirectory that has files
        let tree_with_sub_files = DirectoryTree {
            path: "/test".to_string(),
            files: vec![],
            subdirectories: vec![tree_with_files],
        };
        assert!(has_fmemo_files(&tree_with_sub_files));

        // Tree with no files
        let tree_empty = DirectoryTree {
            path: "/test".to_string(),
            files: vec![],
            subdirectories: vec![],
        };
        assert!(!has_fmemo_files(&tree_empty));
    }

    #[tokio::test]
    async fn test_api_root_endpoint() {
        let temp_dir = TempDir::new().unwrap();
        create_test_fmemo_file(temp_dir.path(), "test1", "# Test 1\nContent 1");
        create_test_fmemo_file(temp_dir.path(), "test2", "# Test 2\nContent 2");

        let api = create_api_routes(temp_dir.path().to_path_buf());

        let response = warp::test::request()
            .method("GET")
            .path("/api/root")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        let files = body["files"].as_array().unwrap();
        assert_eq!(files.len(), 2);
        
        let file_names: Vec<&str> = files.iter()
            .map(|f| f.as_str().unwrap())
            .collect();
        assert!(file_names.contains(&"test1.fmemo"));
        assert!(file_names.contains(&"test2.fmemo"));
    }

    #[tokio::test]
    async fn test_api_files_endpoint_success() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
# Test Function
<desc>A test function</desc>

This is content.

```rust
fn test() {}
```
"#;
        create_test_fmemo_file(temp_dir.path(), "test", content);

        let api = create_api_routes(temp_dir.path().to_path_buf());

        let response = warp::test::request()
            .method("GET")
            .path("/api/files/test.fmemo")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 200);
        
        let body: FileContent = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body.memos.len(), 1);
        assert_eq!(body.memos[0].title(), "Test Function");
        assert_eq!(body.memos[0].description(), &Some("A test function".to_string()));
        assert_eq!(body.memos[0].code_blocks().len(), 1);
        assert!(body.last_modified.is_some());
    }

    #[tokio::test]
    async fn test_api_files_endpoint_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let api = create_api_routes(temp_dir.path().to_path_buf());

        let response = warp::test::request()
            .method("GET")
            .path("/api/files/nonexistent.fmemo")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 404);
        
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body["error"].as_str().unwrap().contains("File not found"));
    }

    #[tokio::test]
    async fn test_api_files_endpoint_invalid_extension() {
        let temp_dir = TempDir::new().unwrap();
        // Create a .md file instead of .fmemo
        fs::write(temp_dir.path().join("test.md"), "# Test").unwrap();
        
        let api = create_api_routes(temp_dir.path().to_path_buf());

        let response = warp::test::request()
            .method("GET")
            .path("/api/files/test.md")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 404);
        
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert!(body["error"].as_str().unwrap().contains("Invalid file type"));
    }

    #[tokio::test]
    async fn test_api_root_endpoint_with_subdirectories() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create file in root
        create_test_fmemo_file(temp_dir.path(), "root", "# Root");
        
        // Create subdirectory with files
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        create_test_fmemo_file(&sub_dir, "sub1", "# Sub 1");
        
        let api = create_api_routes(temp_dir.path().to_path_buf());

        let response = warp::test::request()
            .method("GET")
            .path("/api/root")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        let files = body["files"].as_array().unwrap();
        let directories = body["directories"].as_array().unwrap();
        
        assert_eq!(files.len(), 1);
        assert_eq!(directories.len(), 1);
        assert_eq!(directories[0].as_str().unwrap(), "subdir");
    }

    #[tokio::test] 
    async fn test_api_wrong_method() {
        let temp_dir = TempDir::new().unwrap();
        let api = create_api_routes(temp_dir.path().to_path_buf());

        let response = warp::test::request()
            .method("POST")  // Should be GET
            .path("/api/root")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 405);  // Method Not Allowed
    }

    #[tokio::test]
    async fn test_api_invalid_path() {
        let temp_dir = TempDir::new().unwrap();
        let api = create_api_routes(temp_dir.path().to_path_buf());

        let response = warp::test::request()
            .method("GET")
            .path("/api/invalid")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 404);  // Not Found
    }

    #[tokio::test]
    async fn test_file_change_websocket_integration() {
        use std::time::Duration;
        use tokio::time::timeout;
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_fmemo_file(temp_dir.path(), "test", "# Original Title\nOriginal content");
        
        // Create mock WebSocket client
        let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel();
        let clients: WebSocketClients = Arc::new(Mutex::new(vec![client_tx]));
        
        // Start file watcher
        start_file_watcher(&file_path, clients.clone()).unwrap();
        
        // Give the watcher a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Modify the file
        fs::write(&file_path, "# Updated Title\n<desc>New description</desc>\nUpdated content\n\n```rust\nfn test() {}\n```").unwrap();
        
        // Wait for notification with timeout
        let result = timeout(Duration::from_secs(2), client_rx.recv()).await;
        
        assert!(result.is_ok(), "Should receive WebSocket message within timeout");
        
        let message = result.unwrap().unwrap();
        let text = message.to_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
        
        // Verify message structure
        assert_eq!(parsed["type"], "file_updated");
        assert!(parsed["file_path"].as_str().unwrap().ends_with("test.fmemo"));
        
        // Verify memo content
        let memos = &parsed["memos"];
        assert_eq!(memos.as_array().unwrap().len(), 1);
        
        let memo = &memos[0];
        assert_eq!(memo["title"], "Updated Title");
        assert_eq!(memo["description"], "New description");
        assert_eq!(memo["code_blocks"].as_array().unwrap().len(), 1);
        assert_eq!(memo["code_blocks"][0]["language"], "rust");
    }

    #[tokio::test]
    async fn test_directory_change_websocket_integration() {
        use std::time::Duration;
        use tokio::time::timeout;
        
        let temp_dir = TempDir::new().unwrap();
        create_test_fmemo_file(temp_dir.path(), "existing", "# Existing File");
        
        // Create mock WebSocket client
        let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel();
        let clients: WebSocketClients = Arc::new(Mutex::new(vec![client_tx]));
        
        // Start directory watcher
        start_directory_watcher(temp_dir.path(), clients.clone()).unwrap();
        
        // Give the watcher a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Create a new .fmemo file
        create_test_fmemo_file(temp_dir.path(), "new_file", "# New File\nThis is new content");
        
        // Wait for notification with timeout
        let result = timeout(Duration::from_secs(2), client_rx.recv()).await;
        
        assert!(result.is_ok(), "Should receive WebSocket message within timeout");
        
        let message = result.unwrap().unwrap();
        let text = message.to_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
        
        // Verify message structure
        assert_eq!(parsed["type"], "directory_updated");
        
        // Verify directory tree content
        let tree = &parsed["tree"];
        assert_eq!(tree["files"].as_array().unwrap().len(), 2);
        
        let files = tree["files"].as_array().unwrap();
        let file_names: Vec<&str> = files.iter()
            .map(|f| f.as_str().unwrap())
            .collect();
        
        assert!(file_names.contains(&"existing.fmemo"));
        assert!(file_names.contains(&"new_file.fmemo"));
    }

    #[tokio::test]
    async fn test_multiple_clients_receive_updates() {
        use std::time::Duration;
        use tokio::time::timeout;
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_fmemo_file(temp_dir.path(), "test", "# Test");
        
        // Create multiple mock WebSocket clients
        let (client1_tx, mut client1_rx) = tokio::sync::mpsc::unbounded_channel();
        let (client2_tx, mut client2_rx) = tokio::sync::mpsc::unbounded_channel();
        let clients: WebSocketClients = Arc::new(Mutex::new(vec![client1_tx, client2_tx]));
        
        // Start file watcher
        start_file_watcher(&file_path, clients.clone()).unwrap();
        
        // Give the watcher a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Modify the file
        fs::write(&file_path, "# Updated Content").unwrap();
        
        // Both clients should receive the message
        let result1 = timeout(Duration::from_secs(2), client1_rx.recv()).await;
        let result2 = timeout(Duration::from_secs(2), client2_rx.recv()).await;
        
        assert!(result1.is_ok(), "Client 1 should receive message");
        assert!(result2.is_ok(), "Client 2 should receive message");
        
        // Both should have the same content
        let message1 = result1.unwrap().unwrap();
        let message2 = result2.unwrap().unwrap();
        let msg1 = message1.to_str().unwrap();
        let msg2 = message2.to_str().unwrap();
        assert_eq!(msg1, msg2);
        
        let parsed: serde_json::Value = serde_json::from_str(msg1).unwrap();
        assert_eq!(parsed["type"], "file_updated");
        assert_eq!(parsed["memos"][0]["title"], "Updated Content");
    }

    #[tokio::test]
    async fn test_static_routes_creation() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create mock dist directory structure
        let dist_dir = temp_dir.path().join("dist");
        fs::create_dir(&dist_dir).unwrap();
        fs::create_dir(&dist_dir.join("assets")).unwrap();
        fs::write(&dist_dir.join("index.html"), "<!DOCTYPE html><html><head></head><body></body></html>").unwrap();
        fs::write(&dist_dir.join("favicon.ico"), "fake favicon").unwrap();
        fs::write(&dist_dir.join("vite.svg"), "<svg></svg>").unwrap();
        fs::write(&dist_dir.join("assets").join("main.js"), "console.log('test');").unwrap();
        
        let static_routes = create_static_routes(dist_dir.clone());
        
        // Test serving index.html for SPA routes
        let response = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&static_routes)
            .await;
        
        assert_eq!(response.status(), 200);
        assert!(response.body().starts_with(b"<!DOCTYPE html"));
        
        // Test serving favicon
        let response = warp::test::request()
            .method("GET")
            .path("/favicon.ico")
            .reply(&static_routes)
            .await;
        
        assert_eq!(response.status(), 200);
        assert_eq!(response.body(), &b"fake favicon"[..]);
        
        // Test serving assets
        let response = warp::test::request()
            .method("GET")
            .path("/assets/main.js")
            .reply(&static_routes)
            .await;
        
        assert_eq!(response.status(), 200);
        assert_eq!(response.body(), &b"console.log('test');"[..]);
    }

    #[tokio::test]
    async fn test_api_only_routes() {
        let temp_dir = TempDir::new().unwrap();
        create_test_fmemo_file(temp_dir.path(), "test", "# Test Content\nSome content");
        
        let clients: WebSocketClients = Arc::new(Mutex::new(Vec::new()));
        let api_routes = create_api_only_routes(temp_dir.path().to_path_buf(), clients);
        
        // Test API endpoint works
        let response = warp::test::request()
            .method("GET")
            .path("/api/root")
            .reply(&api_routes)
            .await;
        
        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        let files = body["files"].as_array().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_str().unwrap(), "test.fmemo");
    }

    #[tokio::test]
    async fn test_full_routes_combination() {
        let temp_dir = TempDir::new().unwrap();
        create_test_fmemo_file(temp_dir.path(), "test", "# Test Content\nSome content");
        
        // Create mock dist directory
        let dist_dir = temp_dir.path().join("dist");
        fs::create_dir(&dist_dir).unwrap();
        fs::write(&dist_dir.join("index.html"), "<!DOCTYPE html><html><head></head><body></body></html>").unwrap();
        
        let clients: WebSocketClients = Arc::new(Mutex::new(Vec::new()));
        let full_routes = create_full_routes(
            temp_dir.path().to_path_buf(),
            dist_dir.clone(),
            clients
        );
        
        // Test API endpoint works
        let response = warp::test::request()
            .method("GET")
            .path("/api/root")
            .reply(&full_routes)
            .await;
        
        assert_eq!(response.status(), 200);
        
        // Test static file serving works
        let response = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&full_routes)
            .await;
        
        assert_eq!(response.status(), 200);
        assert!(response.body().starts_with(b"<!DOCTYPE html"));
        
        // Test SPA fallback for unknown routes
        let response = warp::test::request()
            .method("GET")
            .path("/some/spa/route")
            .reply(&full_routes)
            .await;
        
        assert_eq!(response.status(), 200);
        assert!(response.body().starts_with(b"<!DOCTYPE html"));
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let temp_dir = TempDir::new().unwrap();
        create_test_fmemo_file(temp_dir.path(), "test", "# Test Content");
        
        let api_routes = create_api_routes(temp_dir.path().to_path_buf());
        
        let response = warp::test::request()
            .method("GET")
            .path("/api/root")
            .header("Origin", "http://localhost:3000")
            .reply(&api_routes)
            .await;
        
        assert_eq!(response.status(), 200);
        
        // Check CORS headers are present
        let headers = response.headers();
        assert!(headers.contains_key("access-control-allow-origin"));
    }

    #[tokio::test]
    async fn test_api_file_endpoint_frontend_compatible() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
# Test Function
<desc>A test function</desc>

This is content.

```rust
fn test() {}
```
"#;
        create_test_fmemo_file(temp_dir.path(), "test", content);

        let api = create_api_routes(temp_dir.path().to_path_buf());

        let response = warp::test::request()
            .method("GET")
            .path("/api/file/test.fmemo")
            .reply(&api)
            .await;

        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body["path"].as_str().unwrap(), "test.fmemo");
        assert!(body["content"].as_str().is_some());
        assert!(body["memos"].as_array().is_some());
        
        let memos = body["memos"].as_array().unwrap();
        assert_eq!(memos.len(), 1);
        assert_eq!(memos[0]["title"].as_str().unwrap(), "Test Function");
    }
}
