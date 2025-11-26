use clap::{Arg, Command};
use fmemo::server::{create_full_routes, create_api_only_routes, start_directory_watcher, WebSocketClients};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("fmemo")
        .version("0.1.0")
        .about("Real-time Markdown memo server with React frontend")
        .arg(
            Arg::new("root")
                .short('r')
                .long("root")
                .value_name("ROOT_DIR")
                .help("Root directory to serve .fmemo files from")
                .default_value("."),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Port to serve on")
                .default_value("3030"),
        )
        .arg(
            Arg::new("frontend")
                .short('f')
                .long("frontend")
                .value_name("FRONTEND_DIR")
                .help("Frontend dist directory (optional)")
                .required(false),
        )
        .arg(
            Arg::new("api-only")
                .long("api-only")
                .help("Run API server only, without frontend hosting")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dev")
                .long("dev")
                .help("Development mode - serve API only, frontend runs separately on different port")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let root_dir = PathBuf::from(matches.get_one::<String>("root").unwrap());
    let port: u16 = matches
        .get_one::<String>("port")
        .unwrap()
        .parse()
        .expect("Port must be a valid number");
    let frontend_dir = matches.get_one::<String>("frontend").map(PathBuf::from);
    let api_only = matches.get_flag("api-only");
    let dev_mode = matches.get_flag("dev");

    // Validate root directory
    if !root_dir.exists() || !root_dir.is_dir() {
        eprintln!("Error: Root directory '{}' does not exist or is not a directory", root_dir.display());
        std::process::exit(1);
    }

    // Create WebSocket client manager
    let clients: WebSocketClients = Arc::new(Mutex::new(Vec::new()));

    // Start directory watcher for real-time updates
    if let Err(e) = start_directory_watcher(&root_dir, clients.clone()) {
        eprintln!("Warning: Failed to start directory watcher: {}", e);
    }

    // Create routes based on configuration
    if api_only || dev_mode {
        let mode_str = if dev_mode { "development API" } else { "API-only" };
        println!("Starting {} server...", mode_str);
        let routes = create_api_only_routes(root_dir.clone(), clients);
        let routes = routes.with(warp::log("fmemo"));

        println!("Root directory: {}", root_dir.display());
        println!("Server running on http://localhost:{}", port);
        println!("API endpoints:");
        println!("  GET /api/root - Get directory tree");
        println!("  GET /api/files/{{filename}} - Get file content");
        println!("  GET /api/file/{{filename}} - Get file content (frontend compatible)");
        println!("  WebSocket /ws - Real-time updates");
        
        if dev_mode {
            println!("");
            println!("🔧 Development mode:");
            println!("   Run React dev server separately: cd frontend && npm run dev");
            println!("   React dev server will proxy API calls to this server");
            println!("   Configure Vite proxy in vite.config.ts to point to localhost:{}", port);
        }
        
        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    } else if let Some(frontend_path) = frontend_dir {
        if !frontend_path.exists() || !frontend_path.is_dir() {
            eprintln!("Error: Frontend directory '{}' does not exist or is not a directory", frontend_path.display());
            std::process::exit(1);
        }
        println!("Starting server with React frontend...");
        println!("Frontend directory: {}", frontend_path.display());
        let routes = create_full_routes(root_dir.clone(), frontend_path, clients);
        let routes = routes.with(warp::log("fmemo"));

        println!("Root directory: {}", root_dir.display());
        println!("Server running on http://localhost:{}", port);
        println!("Frontend available at: http://localhost:{}/", port);
        println!("API endpoints:");
        println!("  GET /api/root - Get directory tree");
        println!("  GET /api/files/{{filename}} - Get file content");
        println!("  WebSocket /ws - Real-time updates");
        
        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    } else {
        // If compiled with embedded frontend, serve it from the binary
        #[cfg(feature = "embed_frontend")]
        {
            println!("Serving embedded frontend (single binary mode)...");
            let routes = fmemo::server::create_full_routes_embedded(root_dir.clone(), clients.clone());
            let routes = routes.with(warp::log("fmemo"));

            println!("Root directory: {}", root_dir.display());
            println!("Server running on http://localhost:{}", port);
            println!("Frontend available at: http://localhost:{}/", port);
            println!("API endpoints:");
            println!("  GET /api/root - Get directory tree");
            println!("  GET /api/files/{{filename}} - Get file content");
            println!("  WebSocket /ws - Real-time updates");

            warp::serve(routes).run(([127, 0, 0, 1], port)).await;
        }

        // Try to auto-detect frontend directory
        #[cfg(not(feature = "embed_frontend"))]
        {
            let auto_frontend = PathBuf::from("frontend/dist");
            if auto_frontend.exists() && auto_frontend.is_dir() {
                println!("Auto-detected frontend directory: {}", auto_frontend.display());
                let routes = create_full_routes(root_dir.clone(), auto_frontend, clients);
                let routes = routes.with(warp::log("fmemo"));

                println!("Root directory: {}", root_dir.display());
                println!("Server running on http://localhost:{}", port);
                println!("Frontend available at: http://localhost:{}/", port);
                println!("API endpoints:");
                println!("  GET /api/root - Get directory tree");
                println!("  GET /api/files/{{filename}} - Get file content");
                println!("  WebSocket /ws - Real-time updates");

                warp::serve(routes).run(([127, 0, 0, 1], port)).await;
            } else {
                println!("No frontend directory found, starting API-only server...");
                let routes = create_api_only_routes(root_dir.clone(), clients);
                let routes = routes.with(warp::log("fmemo"));

                println!("Root directory: {}", root_dir.display());
                println!("Server running on http://localhost:{}", port);
                println!("API endpoints:");
                println!("  GET /api/root - Get directory tree");
                println!("  GET /api/files/{{filename}} - Get file content");
                println!("  WebSocket /ws - Real-time updates");

                warp::serve(routes).run(([127, 0, 0, 1], port)).await;
            }
        }
    }

    Ok(())
}
