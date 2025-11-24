use crate::parser::parse_memo;
use crate::schema::{DirectoryTree, FileContent};
use std::fs;
use std::path::{Path, PathBuf};
use warp::Filter;

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
                if ext == "fmemo" {
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
    
    // Verify it's a .fmemo file
    if file_path.extension().and_then(|s| s.to_str()) != Some("fmemo") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "File must have .fmemo extension",
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
                            std::io::ErrorKind::InvalidInput => "Invalid file type (must be .fmemo)",
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

    root_route.or(files_route)
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
        
        let body: DirectoryTree = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body.files.len(), 2);
        assert!(body.files.contains(&"test1.fmemo".to_string()));
        assert!(body.files.contains(&"test2.fmemo".to_string()));
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
        
        let body: DirectoryTree = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body.files.len(), 1);
        assert_eq!(body.subdirectories.len(), 1);
        assert_eq!(body.subdirectories[0].files.len(), 1);
        assert_eq!(body.subdirectories[0].files[0], "sub1.fmemo");
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
}