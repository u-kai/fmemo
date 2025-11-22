use std::fs;
use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use std::sync::mpsc::channel;
use std::thread;
use warp::Filter;
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use clap::{Arg, Command};

#[derive(Debug, Clone)]
struct FunctionMemo {
    level: u8,
    title: String,
    content: String,
    code_blocks: Vec<CodeBlock>,
    children: Vec<FunctionMemo>,
}

#[derive(Debug, Clone)]
struct CodeBlock {
    language: String,
    code: String,
}

struct MarkdownParser {
    content: String,
}

impl MarkdownParser {
    fn new(content: String) -> Self {
        Self { content }
    }

    fn parse(&self) -> Vec<FunctionMemo> {
        let flat_memos = self.parse_flat();
        self.build_hierarchy(flat_memos)
    }

    fn parse_flat(&self) -> Vec<FunctionMemo> {
        let mut memos = Vec::new();
        let mut current_memo: Option<FunctionMemo> = None;
        let mut in_code_block = false;
        let mut current_code = String::new();
        let mut current_lang = String::new();
        let mut current_content = String::new();

        for line in self.content.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    if let Some(ref mut memo) = current_memo {
                        memo.code_blocks.push(CodeBlock {
                            language: current_lang.clone(),
                            code: current_code.trim().to_string(),
                        });
                    }
                    current_code.clear();
                    current_lang.clear();
                    in_code_block = false;
                } else {
                    current_lang = line[3..].to_string();
                    in_code_block = true;
                }
            } else if in_code_block {
                current_code.push_str(line);
                current_code.push('\n');
            } else if line.starts_with('#') {
                if let Some(memo) = current_memo.take() {
                    memos.push(memo);
                }
                
                let level = line.chars().take_while(|&c| c == '#').count() as u8;
                let title = line[level as usize..].trim().to_string();
                
                current_memo = Some(FunctionMemo {
                    level,
                    title,
                    content: String::new(),
                    code_blocks: Vec::new(),
                    children: Vec::new(),
                });
                current_content.clear();
            } else if !line.trim().is_empty() {
                current_content.push_str(line);
                current_content.push('\n');
                if let Some(ref mut memo) = current_memo {
                    memo.content = current_content.clone();
                }
            }
        }
        
        if let Some(memo) = current_memo {
            memos.push(memo);
        }
        
        memos
    }

    fn build_hierarchy(&self, flat_memos: Vec<FunctionMemo>) -> Vec<FunctionMemo> {
        let mut root_memos = Vec::new();
        let mut stack: Vec<FunctionMemo> = Vec::new();

        for memo in flat_memos {
            // Pop stack until we find a parent or reach the root
            while let Some(last) = stack.last() {
                if last.level < memo.level {
                    break;
                }
                let completed = stack.pop().unwrap();
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(completed);
                } else {
                    root_memos.push(completed);
                }
            }
            
            stack.push(memo);
        }

        // Process remaining items in stack
        while let Some(memo) = stack.pop() {
            if let Some(parent) = stack.last_mut() {
                parent.children.push(memo);
            } else {
                root_memos.push(memo);
            }
        }

        root_memos
    }
}

struct HtmlGenerator;

impl HtmlGenerator {
    fn generate(memos: &[FunctionMemo], port: u16) -> String {
        let mut html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Function Memo</title>
    <style>
        body {{
            font-family: monospace;
            margin: 20px;
            background-color: #f5f5f5;
            padding: 20px;
        }}
        .memo-container {{
            border: 3px solid #333;
            margin: 15px;
            background-color: white;
            position: relative;
            border-radius: 8px;
            min-height: 60px;
        }}
        .memo-header {{
            padding: 15px 20px 10px 20px;
            border-bottom: 1px solid #eee;
            background-color: #fafafa;
            border-radius: 5px 5px 0 0;
        }}
        .memo-title {{
            font-weight: bold;
            margin: 0;
        }}
        .memo-content {{
            padding: 15px 20px;
            line-height: 1.6;
        }}
        .memo-body {{
            padding: 0px 20px 20px 20px;
        }}
        .code-block {{
            background-color: #f8f8f8;
            border: 1px solid #ddd;
            padding: 15px;
            margin: 15px 0;
            border-radius: 6px;
            overflow-x: auto;
            font-family: 'Courier New', monospace;
            font-size: 0.9em;
        }}
        .code-block pre {{
            margin: 0;
            white-space: pre-wrap;
        }}
        .children-container {{
            margin: 20px 15px 15px 15px;
        }}
        
        /* Level-specific styling */
        .level-1 {{
            border-color: #e74c3c;
            border-width: 4px;
            font-size: 1.1em;
        }}
        .level-1 .memo-title {{ font-size: 1.4em; }}
        
        .level-2 {{
            border-color: #3498db;
            border-width: 3px;
            font-size: 1.05em;
        }}
        .level-2 .memo-title {{ font-size: 1.2em; }}
        
        .level-3 {{
            border-color: #2ecc71;
            border-width: 3px;
        }}
        .level-3 .memo-title {{ font-size: 1.1em; }}
        
        .level-4 {{
            border-color: #f39c12;
            border-width: 2px;
            font-size: 0.95em;
        }}
        .level-4 .memo-title {{ font-size: 1.05em; }}
        
        .level-5 {{
            border-color: #9b59b6;
            border-width: 2px;
            font-size: 0.9em;
        }}
        .level-5 .memo-title {{ font-size: 1.0em; }}
        
        /* Deeper levels get smaller and more subtle */
        .level-6, .level-7, .level-8 {{
            border-color: #95a5a6;
            border-width: 1px;
            font-size: 0.85em;
        }}
        
        /* Collapsible functionality */
        .memo-header {{
            cursor: pointer;
            user-select: none;
            transition: background-color 0.2s;
        }}
        .memo-header:hover {{
            background-color: #f0f0f0 !important;
        }}
        .memo-title-container {{
            display: flex;
            align-items: center;
            gap: 8px;
        }}
        .expand-icon {{
            font-size: 0.8em;
            color: #666;
            transition: transform 0.3s;
            user-select: none;
        }}
        .expand-icon.expanded {{
            transform: rotate(90deg);
        }}
        .children-container {{
            overflow: hidden;
            transition: max-height 0.3s ease-out;
        }}
        .children-container.collapsed {{
            max-height: 0;
        }}
        .children-container.expanded {{
            max-height: 10000px; /* Large enough to show content */
        }}
        .no-children .expand-icon {{
            visibility: hidden;
        }}
    </style>
    <script>
        const ws = new WebSocket('ws://localhost:{}/ws');
        ws.onmessage = function(event) {{
            const data = JSON.parse(event.data);
            if (data.type === 'reload') {{
                location.reload();
            }} else if (data.type === 'update') {{
                updateContent(data.html);
            }}
        }};
        
        function updateContent(newHtml) {{
            const newDoc = new DOMParser().parseFromString(newHtml, 'text/html');
            const newBody = newDoc.body;
            const currentBody = document.body;
            
            // Store current expand/collapse states
            const expandedStates = new Map();
            document.querySelectorAll('.children-container').forEach((container, index) => {{
                expandedStates.set(index, container.classList.contains('expanded'));
            }});
            
            // Replace body content
            currentBody.innerHTML = newBody.innerHTML;
            
            // Restore expand/collapse states
            document.querySelectorAll('.children-container').forEach((container, index) => {{
                if (expandedStates.has(index)) {{
                    const wasExpanded = expandedStates.get(index);
                    if (wasExpanded) {{
                        container.classList.remove('collapsed');
                        container.classList.add('expanded');
                        const header = container.previousElementSibling;
                        if (header && header.classList.contains('memo-header')) {{
                            const icon = header.querySelector('.expand-icon');
                            if (icon) icon.classList.add('expanded');
                        }}
                    }} else {{
                        container.classList.add('collapsed');
                        container.classList.remove('expanded');
                    }}
                }} else {{
                    container.classList.add('collapsed');
                }}
            }});
        }}"#, port);
        
        html.push_str(r#"
        function toggleMemo(element) {
            const container = element.parentElement;
            const childrenContainer = container.querySelector('.children-container');
            const icon = element.querySelector('.expand-icon');
            
            if (childrenContainer) {
                const isExpanded = childrenContainer.classList.contains('expanded');
                
                if (isExpanded) {
                    childrenContainer.classList.remove('expanded');
                    childrenContainer.classList.add('collapsed');
                    icon.classList.remove('expanded');
                } else {
                    childrenContainer.classList.remove('collapsed');
                    childrenContainer.classList.add('expanded');
                    icon.classList.add('expanded');
                }
            }
        }
        
        // Initialize all children as collapsed on page load
        document.addEventListener('DOMContentLoaded', function() {
            const childrenContainers = document.querySelectorAll('.children-container');
            childrenContainers.forEach(container => {
                container.classList.add('collapsed');
            });
        });
    </script>
</head>
<body>
"#);

        for memo in memos {
            Self::generate_memo(&mut html, memo);
        }

        html.push_str("</body>\n</html>");
        html
    }

    pub fn generate_memo(html: &mut String, memo: &FunctionMemo) {
        let level_class = format!("level-{}", memo.level.min(8));
        let has_children = !memo.children.is_empty();
        let no_children_class = if has_children { "" } else { " no-children" };
        
        html.push_str(&format!(
            r#"<div class="memo-container {}{}">
            <div class="memo-header" onclick="toggleMemo(this)">
                <div class="memo-title-container">
                    <span class="expand-icon">▶</span>
                    <h{} class="memo-title">{}</h{}>
                </div>
            </div>
"#,
            level_class,
            no_children_class,
            memo.level.min(6),
            Self::escape_html(&memo.title),
            memo.level.min(6)
        ));

        if !memo.content.trim().is_empty() {
            html.push_str(&format!(
                r#"<div class="memo-content">{}</div>
"#,
                Self::escape_html(&memo.content)
            ));
        }

        // Add code blocks
        if !memo.code_blocks.is_empty() {
            html.push_str("<div class=\"memo-body\">");
            for code_block in &memo.code_blocks {
                html.push_str(&format!(
                    r#"<div class="code-block">
                    <pre><code>{}</code></pre>
                    </div>
"#,
                    Self::escape_html(&code_block.code)
                ));
            }
            html.push_str("</div>");
        }

        // Add children recursively
        if !memo.children.is_empty() {
            html.push_str("<div class=\"children-container\">");
            for child in &memo.children {
                Self::generate_memo(html, child);
            }
            html.push_str("</div>");
        }

        html.push_str("</div>\n");
    }

    fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("fmemo")
        .version("0.1.0")
        .about("Real-time Markdown memo viewer with hierarchical structure")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Markdown file to watch")
                .required(true)
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Port to serve on")
                .default_value("3030")
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").unwrap();
    let port: u16 = matches.get_one::<String>("port").unwrap().parse()
        .expect("Port must be a valid number");

    // Check if file exists
    if !std::path::Path::new(file_path).exists() {
        eprintln!("Error: File '{}' does not exist", file_path);
        std::process::exit(1);
    }

    let html_content = Arc::new(Mutex::new(String::new()));
    let html_content_clone = Arc::clone(&html_content);

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    watcher.watch(std::path::Path::new(file_path), RecursiveMode::NonRecursive)?;

    let websocket_clients: Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<warp::ws::Message>>>> = Arc::new(Mutex::new(Vec::new()));
    let websocket_clients_clone = Arc::clone(&websocket_clients);

    let file_path_clone = file_path.clone();
    let port_clone = port;
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(_event) => {
                    if let Ok(content) = fs::read_to_string(&file_path_clone) {
                        let parser = MarkdownParser::new(content);
                        let memos = parser.parse();
                        let html = HtmlGenerator::generate(&memos, port_clone);
                        
                        {
                            let mut html_guard = html_content_clone.lock().unwrap();
                            *html_guard = html;
                        }

                        let clients = websocket_clients_clone.lock().unwrap();
                        let update_msg = serde_json::json!({
                            "type": "update",
                            "html": format!("<body>{}</body>", 
                                memos.iter()
                                    .map(|memo| {
                                        let mut memo_html = String::new();
                                        HtmlGenerator::generate_memo(&mut memo_html, memo);
                                        memo_html
                                    })
                                    .collect::<Vec<String>>()
                                    .join("")
                            )
                        });
                        
                        for client in clients.iter() {
                            let _ = client.send(warp::ws::Message::text(update_msg.to_string()));
                        }
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    if let Ok(content) = fs::read_to_string(file_path) {
        let parser = MarkdownParser::new(content);
        let memos = parser.parse();
        let html = HtmlGenerator::generate(&memos, port);
        *html_content.lock().unwrap() = html;
    }

    let html_route = {
        let html_content = Arc::clone(&html_content);
        warp::get()
            .and(warp::path::end())
            .map(move || {
                let content = html_content.lock().unwrap().clone();
                warp::reply::html(content)
            })
    };

    let websocket_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let clients = Arc::clone(&websocket_clients);
            ws.on_upgrade(move |websocket| async move {
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
            })
        });

    let routes = html_route.or(websocket_route);

    println!("Server running on http://localhost:{}", port);
    println!("Watching file: {}", file_path);
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await;

    Ok(())
}
