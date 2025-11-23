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
            } else {
                // 空行も含めて全ての行を追加（Markdownの構造を保持）
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
    fn generate(memos: &[FunctionMemo], port: u16, is_horizontal: bool) -> String {
        let mut html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Function Memo</title>
    <style>
        body {{
            font-family: monospace;
            margin: 0;
            background-color: #f5f5f5;
            overflow: auto;
            height: 100vh;
            width: 100vw;
            position: relative;
        }}
        #zoom-container {{
            transform-origin: 0 0;
            transition: transform 0.2s ease-out;
            min-width: fit-content;
            padding: 20px;
            position: absolute;
            top: 0;
            left: 0;
        }}
        #zoom-controls {{
            position: fixed;
            top: 20px;
            left: 20px;
            z-index: 1000;
            background: white;
            padding: 10px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            display: flex;
            gap: 10px;
            align-items: center;
        }}
        .zoom-btn {{
            padding: 8px 12px;
            border: 2px solid #3498db;
            background: white;
            color: #3498db;
            border-radius: 5px;
            cursor: pointer;
            font-family: monospace;
            font-size: 14px;
        }}
        .zoom-btn:hover {{
            background: #3498db;
            color: white;
        }}
        #zoom-level {{
            font-family: monospace;
            font-size: 12px;
            color: #666;
            min-width: 60px;
            text-align: center;
        }}
        .memo-container {{
            border: 3px solid #333;
            margin: 15px;
            background-color: white;
            position: relative;
            border-radius: 8px;
            min-height: 80px;
            min-width: 300px;
            resize: both;
            overflow: auto;
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
        .memo-content p {{
            margin: 0.5em 0;
        }}
        .memo-content ul {{
            margin: 0.5em 0;
            padding-left: 1.5em;
        }}
        .memo-content ul ul {{
            margin: 0.2em 0;
            padding-left: 1.2em;
        }}
        .memo-content li {{
            margin: 0.2em 0;
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
        
        /* Horizontal layout styles */
        .horizontal-layout {{
            display: block;
            min-width: max-content;
        }}
        .horizontal-layout .siblings-container {{
            display: flex;
            flex-wrap: nowrap;
            gap: 25px;
            align-items: flex-start;
            min-width: max-content;
        }}
        .horizontal-layout .memo-container {{
            flex: 0 0 auto;
            min-width: 400px;
            width: 500px;
        }}
        .horizontal-layout .children-container {{
            margin: 25px 20px 20px 20px;
            width: auto;
        }}
        .horizontal-layout .children-container > .memo-container {{
            margin: 20px 0;
            width: 100%;
            min-width: 400px;
        }}
        .horizontal-layout .children-container > .siblings-container {{
            width: 100%;
            min-width: max-content;
        }}
        .horizontal-layout .children-container > .siblings-container > .memo-container {{
            margin: 15px 0;
            min-width: 400px;
            width: 480px;
        }}
        .horizontal-layout .children-container .children-container > .siblings-container > .memo-container {{
            min-width: 350px;
            width: 420px;
        }}
        .horizontal-layout .children-container .children-container .children-container > .siblings-container > .memo-container {{
            min-width: 300px;
            width: 380px;
        }}
        .code-language {{
            background-color: #ddd;
            padding: 2px 8px;
            font-size: 0.8em;
            border-radius: 3px 3px 0 0;
            color: #333;
            font-weight: bold;
        }}
        
        /* Resize handle styling */
        .memo-container::-webkit-resizer {{
            background: linear-gradient(135deg, transparent 40%, #999 40%, #999 60%, transparent 60%);
            border-radius: 0 0 8px 0;
        }}
        
        /* Better resize handle for all browsers */
        .memo-container::after {{
            content: '';
            position: absolute;
            bottom: 0;
            right: 0;
            width: 20px;
            height: 20px;
            background: linear-gradient(135deg, transparent 40%, #666 40%, #666 45%, transparent 45%, transparent 55%, #666 55%, #666 60%, transparent 60%);
            cursor: se-resize;
            border-radius: 0 0 8px 0;
            pointer-events: none;
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
        
        html.push_str("
        function toggleMemo(element) {
            const container = element.parentElement;
            const childrenContainer = container.querySelector('.children-container');
            const icon = element.querySelector('.expand-icon');
            
            if (childrenContainer) {
                const isExpanded = childrenContainer.classList.contains('expanded');
                
                if (isExpanded) {
                    // Collapsing - restore original size if no manual resize
                    childrenContainer.classList.remove('expanded');
                    childrenContainer.classList.add('collapsed');
                    icon.classList.remove('expanded');
                    
                    // Restore original size if it was auto-resized
                    restoreOriginalSize(container);
                } else {
                    // Expanding - save original size first
                    saveOriginalSize(container);
                    
                    childrenContainer.classList.remove('collapsed');
                    childrenContainer.classList.add('expanded');
                    icon.classList.add('expanded');
                    
                    // Auto-resize container when expanding children
                    setTimeout(() => {
                        autoResizeContainerRecursive(container);
                    }, 300); // Wait for transition to complete
                }
            }
        }
        
        function saveOriginalSize(container) {
            // Only save if not already saved and not manually resized
            if (!container.dataset.originalWidth && !container.dataset.originalHeight) {
                const rect = container.getBoundingClientRect();
                const computedStyle = window.getComputedStyle(container);
                
                // Save original dimensions
                container.dataset.originalWidth = rect.width;
                container.dataset.originalHeight = rect.height;
                container.dataset.hadManualResize = container.style.width ? 'true' : 'false';
            }
        }
        
        function restoreOriginalSize(container) {
            // Only restore if we have saved dimensions and it wasn't manually resized
            if (container.dataset.originalWidth && container.dataset.originalHeight && 
                container.dataset.hadManualResize === 'false') {
                
                // Check if no children are expanded
                const expandedChildren = container.querySelectorAll('.children-container.expanded');
                if (expandedChildren.length === 0) {
                    container.style.width = container.dataset.originalWidth + 'px';
                    container.style.height = container.dataset.originalHeight + 'px';
                }
            }
        }
        
        function autoResizeContainerRecursive(container) {
            // First resize this container
            autoResizeContainer(container);
            
            // Then find parent containers and resize them too
            let currentContainer = container;
            while (currentContainer) {
                // Find the parent memo container
                let parentContainer = currentContainer.parentElement;
                while (parentContainer && !parentContainer.classList.contains('memo-container')) {
                    parentContainer = parentContainer.parentElement;
                }
                
                if (parentContainer) {
                    // Resize parent based on its children
                    autoResizeContainer(parentContainer);
                    currentContainer = parentContainer;
                } else {
                    break;
                }
            }
        }
        
        function autoResizeContainer(container) {
            // Check if container has been manually resized (has inline styles)
            const hasManualWidth = container.style.width && container.style.width !== '';
            const hasManualHeight = container.style.height && container.style.height !== '';
            
            // Get current container size
            const containerRect = container.getBoundingClientRect();
            let requiredWidth = 0;
            let requiredHeight = 0;
            
            // Check if there are expanded children
            const childrenContainer = container.querySelector('.children-container');
            if (childrenContainer && childrenContainer.classList.contains('expanded')) {
                const siblingsContainer = childrenContainer.querySelector('.siblings-container');
                
                if (siblingsContainer) {
                    // Multiple siblings case
                    const children = siblingsContainer.querySelectorAll(':scope > .memo-container');
                    let totalChildWidth = 0;
                    let maxChildHeight = 0;
                    
                    children.forEach(child => {
                        const rect = child.getBoundingClientRect();
                        totalChildWidth += rect.width;
                        maxChildHeight = Math.max(maxChildHeight, rect.height);
                    });
                    
                    // Calculate required width (children + gaps + margins)
                    const gap = 25; // CSS gap
                    const margin = 80; // Container margins (increased)
                    const childrenWidth = totalChildWidth + (gap * (children.length - 1)) + margin;
                    requiredWidth = Math.max(requiredWidth, childrenWidth);
                    
                    // Calculate required height (current + children + buffer)
                    const headerHeight = 100; // Estimate for header and content
                    const childrenHeight = maxChildHeight + headerHeight + 50; // Buffer
                    requiredHeight = Math.max(requiredHeight, childrenHeight);
                    
                } else {
                    // Single child case
                    const child = childrenContainer.querySelector('.memo-container');
                    if (child) {
                        const childRect = child.getBoundingClientRect();
                        requiredWidth = Math.max(requiredWidth, childRect.width + 120);
                        requiredHeight = Math.max(requiredHeight, containerRect.height + childRect.height + 80);
                    }
                }
                
                // Only apply size if children actually need more space
                if (requiredWidth > 0 && requiredHeight > 0) {
                    const currentWidth = containerRect.width;
                    const currentHeight = containerRect.height;
                    
                    // Width adjustment
                    if (!hasManualWidth && requiredWidth > currentWidth) {
                        container.style.width = requiredWidth + 'px';
                    } else if (hasManualWidth && requiredWidth > currentWidth) {
                        const manualWidth = parseInt(container.style.width);
                        if (requiredWidth > manualWidth) {
                            container.style.width = requiredWidth + 'px';
                        }
                    }
                    
                    // Height adjustment  
                    if (!hasManualHeight && requiredHeight > currentHeight) {
                        container.style.height = requiredHeight + 'px';
                    } else if (hasManualHeight && requiredHeight > currentHeight) {
                        const manualHeight = parseInt(container.style.height);
                        if (requiredHeight > manualHeight) {
                            container.style.height = requiredHeight + 'px';
                        }
                    }
                }
            }
        }
        
        // Zoom functionality
        let currentZoom = 1.0;
        let panX = 0;
        let panY = 0;
        let isPanning = false;
        let lastMouseX = 0;
        let lastMouseY = 0;
        
        function updateZoomDisplay() {
            document.getElementById('zoom-level').textContent = Math.round(currentZoom * 100) + '%';
        }
        
        function applyZoom() {
            const container = document.getElementById('zoom-container');
            container.style.transform = `scale(${currentZoom}) translate(${panX}px, ${panY}px)`;
            updateZoomDisplay();
        }
        
        function zoomIn() {
            currentZoom = Math.min(currentZoom * 1.25, 5.0);
            applyZoom();
        }
        
        function zoomOut() {
            currentZoom = Math.max(currentZoom * 0.8, 0.1);
            applyZoom();
        }
        
        function resetZoom() {
            currentZoom = 1.0;
            panX = 0;
            panY = 0;
            applyZoom();
        }
        
        function fitToScreen() {
            const container = document.getElementById('zoom-container');
            const containerRect = container.getBoundingClientRect();
            const windowWidth = window.innerWidth;
            const windowHeight = window.innerHeight;
            
            const scaleX = (windowWidth - 100) / containerRect.width;
            const scaleY = (windowHeight - 100) / containerRect.height;
            currentZoom = Math.min(scaleX, scaleY, 1.0);
            
            panX = 0;
            panY = 0;
            applyZoom();
        }
        
        // Mouse wheel zoom (only with Ctrl key)
        document.addEventListener('wheel', function(e) {
            if (e.ctrlKey || e.metaKey) {
                // Zoom mode
                e.preventDefault();
                
                const delta = e.deltaY > 0 ? 0.9 : 1.1;
                const newZoom = Math.max(Math.min(currentZoom * delta, 5.0), 0.1);
                
                // Zoom toward mouse position
                const rect = document.getElementById('zoom-container').getBoundingClientRect();
                const mouseX = e.clientX - rect.left;
                const mouseY = e.clientY - rect.top;
                
                const zoomRatio = newZoom / currentZoom;
                panX = mouseX - (mouseX - panX) * zoomRatio;
                panY = mouseY - (mouseY - panY) * zoomRatio;
                
                currentZoom = newZoom;
                applyZoom();
            }
            // If no Ctrl key, allow normal scrolling
        });
        
        // Panning with mouse drag
        document.addEventListener('mousedown', function(e) {
            // Don't pan when interacting with UI elements or resize handles
            if (e.target.closest('#zoom-controls') || 
                e.target.closest('.memo-header') ||
                e.target.closest('.memo-container') || 
                e.ctrlKey) {
                return;
            }
            
            // Only pan on background or body
            if (e.target === document.body || e.target === document.getElementById('zoom-container')) {
                isPanning = true;
                lastMouseX = e.clientX;
                lastMouseY = e.clientY;
                document.body.style.cursor = 'grab';
            }
        });
        
        document.addEventListener('mousemove', function(e) {
            if (isPanning) {
                const deltaX = e.clientX - lastMouseX;
                const deltaY = e.clientY - lastMouseY;
                
                panX += deltaX / currentZoom;
                panY += deltaY / currentZoom;
                
                lastMouseX = e.clientX;
                lastMouseY = e.clientY;
                
                applyZoom();
            }
        });
        
        document.addEventListener('mouseup', function() {
            isPanning = false;
            document.body.style.cursor = 'default';
        });
        
        // Keyboard shortcuts
        document.addEventListener('keydown', function(e) {
            if (e.ctrlKey || e.metaKey) {
                switch(e.key) {
                    case '=':
                    case '+':
                        e.preventDefault();
                        zoomIn();
                        break;
                    case '-':
                        e.preventDefault();
                        zoomOut();
                        break;
                    case '0':
                        e.preventDefault();
                        resetZoom();
                        break;
                }
            }
        });
        
        // Initialize all children as collapsed on page load
        document.addEventListener('DOMContentLoaded', function() {
            const childrenContainers = document.querySelectorAll('.children-container');
            childrenContainers.forEach(container => {
                container.classList.add('collapsed');
            });
        });
    </script>
</head>
<body class=\"");
        html.push_str(if is_horizontal { "horizontal-layout" } else { "" });
        html.push_str("\">");
        
        html.push_str(r#"
<div id="zoom-controls">
    <button class="zoom-btn" onclick="zoomOut()">−</button>
    <span id="zoom-level">100%</span>
    <button class="zoom-btn" onclick="zoomIn()">+</button>
    <button class="zoom-btn" onclick="resetZoom()">Reset</button>
    <button class="zoom-btn" onclick="fitToScreen()">Fit</button>
</div>
<div id="zoom-container">
"#);

        if is_horizontal {
            Self::generate_horizontal_layout(&mut html, memos);
        } else {
            for memo in memos {
                Self::generate_memo(&mut html, memo);
            }
        }

        html.push_str("</div></body>\n</html>");
        html
    }

    fn generate_horizontal_layout(html: &mut String, memos: &[FunctionMemo]) {
        // 最上位レベルの兄弟要素を横並びで表示
        html.push_str("<div class=\"siblings-container\">");
        for memo in memos {
            Self::generate_memo_horizontal(html, memo);
        }
        html.push_str("</div>");
    }

    fn generate_memo_horizontal(html: &mut String, memo: &FunctionMemo) {
        let level_class = format!("level-{}", memo.level.min(8));
        let has_children = !memo.children.is_empty();
        let no_children_class = if has_children { "" } else { " no-children" };
        
        // Start container
        html.push_str("<div class=\"memo-container ");
        html.push_str(&level_class);
        html.push_str(no_children_class);
        html.push_str("\">");
        
        // Header
        html.push_str("<div class=\"memo-header\" onclick=\"toggleMemo(this)\">");
        html.push_str("<div class=\"memo-title-container\">");
        html.push_str("<span class=\"expand-icon\">▶</span>");
        html.push_str(&format!("<h{} class=\"memo-title\">", memo.level.min(6)));
        html.push_str(&Self::escape_html(&memo.title));
        html.push_str(&format!("</h{}>", memo.level.min(6)));
        html.push_str("</div>");
        html.push_str("</div>");

        if !memo.content.trim().is_empty() {
            html.push_str("<div class=\"memo-content\">");
            html.push_str(&Self::markdown_to_html(&memo.content));
            html.push_str("</div>");
        }

        // Add code blocks
        if !memo.code_blocks.is_empty() {
            html.push_str("<div class=\"memo-body\">");
            for code_block in &memo.code_blocks {
                html.push_str("<div class=\"code-block\">");
                if !code_block.language.is_empty() {
                    html.push_str(&format!("<div class=\"code-language\">{}</div>", Self::escape_html(&code_block.language)));
                }
                html.push_str("<pre><code>");
                html.push_str(&Self::escape_html(&code_block.code));
                html.push_str("</code></pre></div>");
            }
            html.push_str("</div>");
        }

        // Add children recursively - siblings are always horizontal
        if !memo.children.is_empty() {
            html.push_str("<div class=\"children-container\">");
            if memo.children.len() > 1 {
                // Multiple children = siblings, display horizontally
                html.push_str("<div class=\"siblings-container\">");
                for child in &memo.children {
                    Self::generate_memo_horizontal(html, child);
                }
                html.push_str("</div>");
            } else {
                // Single child, no siblings-container needed
                for child in &memo.children {
                    Self::generate_memo_horizontal(html, child);
                }
            }
            html.push_str("</div>");
        }

        html.push_str("</div>");
    }

    pub fn generate_memo(html: &mut String, memo: &FunctionMemo) {
        let level_class = format!("level-{}", memo.level.min(8));
        let has_children = !memo.children.is_empty();
        let no_children_class = if has_children { "" } else { " no-children" };
        
        // Start container
        html.push_str("<div class=\"memo-container ");
        html.push_str(&level_class);
        html.push_str(no_children_class);
        html.push_str("\">");
        
        // Header
        html.push_str("<div class=\"memo-header\" onclick=\"toggleMemo(this)\">");
        html.push_str("<div class=\"memo-title-container\">");
        html.push_str("<span class=\"expand-icon\">▶</span>");
        html.push_str(&format!("<h{} class=\"memo-title\">", memo.level.min(6)));
        html.push_str(&Self::escape_html(&memo.title));
        html.push_str(&format!("</h{}>", memo.level.min(6)));
        html.push_str("</div>");
        html.push_str("</div>");

        if !memo.content.trim().is_empty() {
            html.push_str("<div class=\"memo-content\">");
            html.push_str(&Self::markdown_to_html(&memo.content));
            html.push_str("</div>");
        }

        // Add code blocks
        if !memo.code_blocks.is_empty() {
            html.push_str("<div class=\"memo-body\">");
            for code_block in &memo.code_blocks {
                html.push_str("<div class=\"code-block\"><pre><code>");
                html.push_str(&Self::escape_html(&code_block.code));
                html.push_str("</code></pre></div>");
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

        html.push_str("</div>");
    }

    fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    fn markdown_to_html(text: &str) -> String {
        let mut html = String::new();
        let mut list_levels: Vec<usize> = Vec::new(); // インデントレベルを追跡
        let mut current_paragraph = String::new();

        for line in text.lines() {
            let leading_spaces = line.len() - line.trim_start().len();
            let trimmed = line.trim();
            
            if trimmed.starts_with("- ") {
                // 段落があれば閉じる
                if !current_paragraph.trim().is_empty() {
                    html.push_str(&format!("<p>{}</p>\n", current_paragraph.trim()));
                    current_paragraph.clear();
                }

                let current_level = leading_spaces;
                
                // リストレベルの調整
                while !list_levels.is_empty() && list_levels.last().unwrap() >= &current_level {
                    list_levels.pop();
                    html.push_str("</ul>\n");
                }
                
                if list_levels.is_empty() || list_levels.last().unwrap() < &current_level {
                    list_levels.push(current_level);
                    html.push_str("<ul>\n");
                }
                
                let item_text = &trimmed[2..]; // "- " を除去
                let indent = "  ".repeat(list_levels.len());
                html.push_str(&format!("{}<li>{}</li>\n", indent, Self::escape_html(item_text)));
            } else if trimmed.is_empty() {
                // 空行の処理
                while !list_levels.is_empty() {
                    list_levels.pop();
                    html.push_str("</ul>\n");
                }
                if !current_paragraph.trim().is_empty() {
                    html.push_str(&format!("<p>{}</p>\n", current_paragraph.trim()));
                    current_paragraph.clear();
                }
            } else {
                // 通常のテキスト行
                while !list_levels.is_empty() {
                    list_levels.pop();
                    html.push_str("</ul>\n");
                }
                if !current_paragraph.is_empty() {
                    current_paragraph.push(' ');
                }
                current_paragraph.push_str(trimmed);
            }
        }

        // 最後の処理
        while !list_levels.is_empty() {
            list_levels.pop();
            html.push_str("</ul>\n");
        }
        if !current_paragraph.trim().is_empty() {
            html.push_str(&format!("<p>{}</p>\n", current_paragraph.trim()));
        }

        html
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
        .arg(
            Arg::new("layout")
                .short('l')
                .long("layout")
                .value_name("LAYOUT")
                .help("Layout mode: vertical (default) or horizontal")
                .default_value("vertical")
                .value_parser(["vertical", "horizontal"])
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").unwrap();
    let port: u16 = matches.get_one::<String>("port").unwrap().parse()
        .expect("Port must be a valid number");
    let layout = matches.get_one::<String>("layout").unwrap();
    let is_horizontal = layout == "horizontal";

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
    let is_horizontal_clone = is_horizontal;
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(_event) => {
                    if let Ok(content) = fs::read_to_string(&file_path_clone) {
                        let parser = MarkdownParser::new(content);
                        let memos = parser.parse();
                        let html = HtmlGenerator::generate(&memos, port_clone, is_horizontal_clone);
                        
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
        let html = HtmlGenerator::generate(&memos, port, is_horizontal);
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
