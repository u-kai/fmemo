use clap::{Arg, Command};
use futures_util::{SinkExt, StreamExt};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use warp::Filter;

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
        let mut html = format!(
            r#"
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
            min-width: max-content;
            min-height: max-content;
            width: max-content;
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
        .control-separator {{
            width: 1px;
            height: 30px;
            background: #ddd;
            margin: 0 10px;
        }}
        .mode-btn {{
            padding: 8px 16px;
            border: 2px solid #2ecc71;
            background: white;
            color: #2ecc71;
            border-radius: 5px;
            cursor: pointer;
            font-family: monospace;
            font-size: 14px;
        }}
        .mode-btn:hover {{
            background: #2ecc71;
            color: white;
        }}
        .mode-btn.active {{
            background: #2ecc71;
            color: white;
        }}
        .memo-container {{
            display: inline-block;
            width: max-content;
            border: 3px solid #333;
            margin: 15px;
            background-color: white;
            position: relative;
            border-radius: 8px;
            min-height: 80px;
            min-width: 260px;
            overflow: visible;
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
            transition: max-height 0.3s ease-out, width 0.3s ease-out, margin 0.3s ease-out;
        }}
        .children-container.collapsed {{
            max-height: 0;
            width: 0 !important;
            min-width: 0 !important;
            margin: 0 !important;
            padding: 0 !important;
        }}
        .children-container.expanded {{
            max-height: 10000px; /* Large enough to show content */
            width: max-content;
            margin: 20px 15px 15px 15px;
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
            display: inline-flex;
            flex-direction: column;
            width: max-content;
            min-width: 280px;
        }}
        .horizontal-layout .children-container.expanded {{
            margin: 25px 20px 20px 20px;
            width: max-content;
        }}
        .horizontal-layout .children-container.collapsed {{
            width: 0 !important;
            min-width: 0 !important;
            margin: 0 !important;
            padding: 0 !important;
        }}
        .horizontal-layout .siblings-container {{
            display: flex;
            flex-wrap: nowrap;
            gap: 25px;
            align-items: flex-start;
            width: max-content;
        }}
        .horizontal-layout .siblings-container > .memo-container {{
            margin: 15px 0;
            min-width: 260px;
        }}
        .code-language {{
            background-color: #ddd;
            padding: 2px 8px;
            font-size: 0.8em;
            border-radius: 3px 3px 0 0;
            color: #333;
            font-weight: bold;
        }}
        
        /* View mode management */
        .view-mode {{
            display: none;
        }}
        .view-mode.active {{
            display: block;
        }}
        
        /* Flow diagram styles */
        #flow-view {{
            padding: 40px 20px;
            width: max-content;
            min-height: 100vh;
            position: relative;
        }}
        .flow-diagram-container {{
            position: relative;
            padding: 20px;
        }}
        .flow-tree-node {{
            margin: 20px 0;
        }}
        .siblings-flow {{
            display: flex;
            gap: 30px;
            align-items: flex-start;
            margin: 20px 0;
        }}
        .flow-tree-node.top-level {{
            margin: 0 40px 60px 40px;
        }}
        .children-flow {{
            margin-top: 40px;
            margin-left: 40px;
        }}
        .flow-node {{
            display: inline-block;
            background: white;
            border: 3px solid;
            border-radius: 10px;
            padding: 12px 16px;
            font-family: monospace;
            font-size: 14px;
            cursor: pointer;
            transition: all 0.3s ease;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            min-width: 100px;
            max-width: 250px;
            width: fit-content;
            text-align: center;
            position: relative;
            margin: 10px;
            white-space: normal;
            word-wrap: break-word;
            overflow-wrap: break-word;
        }}
        .flow-node-title {{
            font-weight: bold;
            margin-bottom: 6px;
        }}
        .flow-node-description {{
            font-size: 11px;
            color: #666;
            font-style: italic;
            line-height: 1.2;
            margin-top: 4px;
            opacity: 0.8;
        }}
        .flow-node-path {{
            font-size: 10px;
            color: #888;
            font-family: 'Courier New', monospace;
            background: rgba(0,0,0,0.05);
            padding: 2px 6px;
            border-radius: 3px;
            margin-top: 4px;
            border: 1px solid rgba(0,0,0,0.1);
            word-break: break-all;
            overflow-wrap: break-word;
        }}
        .flow-node:hover {{
            transform: translateY(-3px);
            box-shadow: 0 6px 20px rgba(0,0,0,0.15);
        }}
        .flow-node.level-1 {{ 
            border-color: #e74c3c; 
            background: #fdf2f2;
            font-size: 16px;
            font-weight: bold;
        }}
        .flow-node.level-2 {{ 
            border-color: #3498db;
            background: #f0f8ff;
            font-size: 15px;
        }}
        .flow-node.level-3 {{ 
            border-color: #2ecc71;
            background: #f0fff4;
        }}
        .flow-node.level-4 {{ 
            border-color: #f39c12;
            background: #fffaf0;
        }}
        .flow-node.level-5 {{ 
            border-color: #9b59b6;
            background: #f8f5ff;
        }}
        .flow-node.level-6,
        .flow-node.level-7,
        .flow-node.level-8 {{ 
            border-color: #95a5a6;
            background: #f8f9fa;
            font-size: 12px;
        }}
        .flow-connection-line {{
            position: absolute;
            z-index: 1;
        }}
        .flow-arrow {{
            position: absolute;
            z-index: 2;
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
            const newMemoView = newDoc.querySelector('#memo-view');
            const currentMemoView = document.getElementById('memo-view');
            
            if (!newMemoView || !currentMemoView) return;
            
            // Store current view mode state
            const isFlowMode = document.getElementById('flow-view').classList.contains('active');
            
            // Store current expand/collapse states
            const expandedStates = new Map();
            document.querySelectorAll('.children-container').forEach((container, index) => {{
                expandedStates.set(index, container.classList.contains('expanded'));
            }});
            
            // Store current zoom and pan state
            const currentZoomState = {{
                zoom: currentZoom,
                panX: panX,
                panY: panY
            }};
            
            // Replace only memo-view content, preserving view mode classes
            currentMemoView.innerHTML = newMemoView.innerHTML;
            
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
            
            // If in flow mode, regenerate flow diagram
            if (isFlowMode) {{
                setTimeout(() => {{
                    generateFlowDiagram();
                }}, 10);
            }}
        }}"#,
            port
        );

        html.push_str("
        function toggleMemo(element) {
            const container = element.parentElement;
            const childrenContainer = container.querySelector('.children-container');
            const icon = element.querySelector('.expand-icon');
            
            if (!childrenContainer) return;

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
        
        // View mode switching
        function switchToMemoMode() {
            document.getElementById('memo-view').classList.add('active');
            document.getElementById('flow-view').classList.remove('active');
            document.getElementById('memo-mode-btn').classList.add('active');
            document.getElementById('flow-mode-btn').classList.remove('active');
        }
        
        function switchToFlowMode() {
            document.getElementById('memo-view').classList.remove('active');
            document.getElementById('flow-view').classList.add('active');
            document.getElementById('memo-mode-btn').classList.remove('active');
            document.getElementById('flow-mode-btn').classList.add('active');
            
            // Generate flow diagram
            generateFlowDiagram();
        }
        
        function generateFlowDiagram() {
            const flowView = document.getElementById('flow-view');
            if (!flowView) return;
            
            // Build hierarchical structure from memo containers
            const hierarchy = buildHierarchyFromDOM();
            
            // Generate hierarchical flow HTML
            let html = '<div class=\"flow-diagram-container\">';
            html += generateHierarchicalFlow(hierarchy, 0);
            html += '</div>';
            
            flowView.innerHTML = html;
            
            // Add connecting lines after DOM is ready
            setTimeout(() => {
                addConnectingLines();
            }, 100);
        }
        
        function buildHierarchyFromDOM() {
            const rootNodes = [];
            const memoView = document.getElementById('memo-view');
            
            // Temporarily make memo-view visible to read DOM structure
            const wasHidden = !memoView.classList.contains('active');
            if (wasHidden) {
                memoView.style.display = 'block';
                memoView.style.visibility = 'hidden';
                memoView.style.position = 'absolute';
                memoView.style.left = '-9999px';
            }
            
            // Get root level memo containers (direct children of memo-view)
            const rootContainers = document.querySelectorAll('#memo-view > .memo-container, #memo-view > .siblings-container > .memo-container');
            
            rootContainers.forEach(container => {
                const node = buildNodeFromContainer(container);
                if (node) {
                    rootNodes.push(node);
                }
            });
            
            // Restore original state
            if (wasHidden) {
                memoView.style.display = '';
                memoView.style.visibility = '';
                memoView.style.position = '';
                memoView.style.left = '';
            }
            
            return rootNodes;
        }
        
        function buildNodeFromContainer(container) {
            const titleElement = container.querySelector('.memo-title');
            if (!titleElement) return null;
            
            let level = 1;
            for (let i = 1; i <= 8; i++) {
                if (container.classList.contains('level-' + i)) {
                    level = i;
                    break;
                }
            }
            
            // Extract description and path from content
            const contentElement = container.querySelector('.memo-content, .memo-body');
            let description = '';
            let path = '';
            
            if (contentElement) {
                const htmlContent = contentElement.innerHTML;
                
                // Extract description from <desc></desc> tags
                const descMatch = htmlContent.match(/<desc>(.*?)<\\/desc>/i);
                if (descMatch) {
                    description = descMatch[1].trim();
                }
                
                // Extract path from <path></path> tags
                const pathMatch = htmlContent.match(/<path>(.*?)<\\/path>/i);
                if (pathMatch) {
                    path = pathMatch[1].trim();
                }
            }
            
            const node = {
                title: titleElement.textContent.trim(),
                level: level,
                description: description,
                path: path,
                children: []
            };
            
            // Find children containers
            const childrenContainer = container.querySelector('.children-container');
            if (childrenContainer) {
                const childContainers = childrenContainer.querySelectorAll(':scope > .memo-container, :scope > .siblings-container > .memo-container');
                childContainers.forEach(childContainer => {
                    const childNode = buildNodeFromContainer(childContainer);
                    if (childNode) {
                        node.children.push(childNode);
                    }
                });
            }
            
            return node;
        }
        
        function generateHierarchicalFlow(nodes, depth) {
            if (!nodes || nodes.length === 0) return '';
            
            let html = '';
            
            if (nodes.length === 1) {
                // Single node
                const node = nodes[0];
                html += '<div class=\"flow-tree-node\">';
                html += generateFlowNode(node, depth);
                if (node.children.length > 0) {
                    html += '<div class=\"children-flow\">';
                    html += generateHierarchicalFlow(node.children, depth + 1);
                    html += '</div>';
                }
                html += '</div>';
            } else {
                // Multiple siblings
                html += '<div class=\"siblings-flow\">';
                nodes.forEach(node => {
                    const isTopLevel = node.level === 1 ? ' top-level' : '';
                    html += '<div class=\"flow-tree-node sibling' + isTopLevel + '\">';
                    html += generateFlowNode(node, depth);
                    if (node.children.length > 0) {
                        html += '<div class=\"children-flow\">';
                        html += generateHierarchicalFlow(node.children, depth + 1);
                        html += '</div>';
                    }
                    html += '</div>';
                });
                html += '</div>';
            }
            
            return html;
        }
        
        function generateFlowNode(node, depth) {
            const safeTitle = node.title.replace(/'/g, '&#39;').replace(/\"/g, '&quot;');
            const safeDescription = (node.description || '').replace(/'/g, '&#39;').replace(/\"/g, '&quot;');
            const safePath = (node.path || '').replace(/'/g, '&#39;').replace(/\"/g, '&quot;');
            
            let html = '<div class=\"flow-node level-' + node.level + ' depth-' + depth + '\" onclick=\"jumpToMemo(\\'' + safeTitle + '\\')\">'; 
            html += '<div class=\"flow-node-title\">' + node.title + '</div>';
            
            if (node.description && node.description.length > 0) {
                html += '<div class=\"flow-node-description\">' + node.description + '</div>';
            }
            
            if (node.path && node.path.length > 0) {
                html += '<div class=\"flow-node-path\">' + node.path + '</div>';
            }
            
            html += '</div>';
            return html;
        }
        
        function addConnectingLines() {
            // Remove existing lines
            document.querySelectorAll('.flow-connection-line').forEach(line => line.remove());
            
            // Add parent-child lines (vertical)
            document.querySelectorAll('.flow-tree-node').forEach(treeNode => {
                const parentNode = treeNode.querySelector(':scope > .flow-node');
                const childrenFlow = treeNode.querySelector(':scope > .children-flow');
                
                if (parentNode && childrenFlow) {
                    const childNodes = childrenFlow.querySelectorAll(':scope > .flow-tree-node > .flow-node, :scope > .siblings-flow > .flow-tree-node > .flow-node');
                    
                    if (childNodes.length > 0) {
                        childNodes.forEach(childNode => {
                            createVerticalLine(parentNode, childNode);
                        });
                    }
                }
            });
            
            // Add sibling lines (horizontal) - skip level 1 siblings
            document.querySelectorAll('.siblings-flow').forEach(siblingsContainer => {
                const siblingNodes = siblingsContainer.querySelectorAll(':scope > .flow-tree-node > .flow-node');
                
                // Check if any of the siblings are level-1 (top level)
                let hasLevel1 = false;
                siblingNodes.forEach(node => {
                    if (node.classList.contains('level-1')) {
                        hasLevel1 = true;
                    }
                });
                
                // Only add horizontal lines if not level-1 siblings
                if (!hasLevel1) {
                    for (let i = 0; i < siblingNodes.length - 1; i++) {
                        createHorizontalLine(siblingNodes[i], siblingNodes[i + 1]);
                    }
                }
            });
        }
        
        function createVerticalLine(parentNode, childNode) {
            const parentRect = parentNode.getBoundingClientRect();
            const childRect = childNode.getBoundingClientRect();
            const flowRect = document.getElementById('flow-view').getBoundingClientRect();
            
            const line = document.createElement('div');
            line.className = 'flow-connection-line vertical-line';
            
            const startX = parentRect.left + parentRect.width / 2 - flowRect.left;
            const startY = parentRect.bottom - flowRect.top;
            const endY = childRect.top - flowRect.top;
            
            line.style.position = 'absolute';
            line.style.left = (startX - 1) + 'px';
            line.style.top = startY + 'px';
            line.style.width = '2px';
            line.style.height = (endY - startY) + 'px';
            line.style.background = '#34495e';
            line.style.zIndex = '1';
            
            document.getElementById('flow-view').appendChild(line);
            
            // Add arrow at the end
            const arrow = document.createElement('div');
            arrow.className = 'flow-arrow down-arrow';
            arrow.style.position = 'absolute';
            arrow.style.left = (startX - 6) + 'px';
            arrow.style.top = (endY - 10) + 'px';
            arrow.style.width = '0';
            arrow.style.height = '0';
            arrow.style.borderLeft = '6px solid transparent';
            arrow.style.borderRight = '6px solid transparent';
            arrow.style.borderTop = '10px solid #34495e';
            arrow.style.zIndex = '2';
            
            document.getElementById('flow-view').appendChild(arrow);
        }
        
        function createHorizontalLine(leftNode, rightNode) {
            const leftRect = leftNode.getBoundingClientRect();
            const rightRect = rightNode.getBoundingClientRect();
            const flowRect = document.getElementById('flow-view').getBoundingClientRect();
            
            const line = document.createElement('div');
            line.className = 'flow-connection-line horizontal-line';
            
            const startX = leftRect.right - flowRect.left;
            const endX = rightRect.left - flowRect.left;
            const y = leftRect.top + leftRect.height / 2 - flowRect.top;
            
            line.style.position = 'absolute';
            line.style.left = startX + 'px';
            line.style.top = (y - 1) + 'px';
            line.style.width = (endX - startX) + 'px';
            line.style.height = '2px';
            line.style.background = '#95a5a6';
            line.style.zIndex = '1';
            
            document.getElementById('flow-view').appendChild(line);
            
            // Add arrow at the end
            const arrow = document.createElement('div');
            arrow.className = 'flow-arrow right-arrow';
            arrow.style.position = 'absolute';
            arrow.style.left = (endX - 10) + 'px';
            arrow.style.top = (y - 6) + 'px';
            arrow.style.width = '0';
            arrow.style.height = '0';
            arrow.style.borderTop = '6px solid transparent';
            arrow.style.borderBottom = '6px solid transparent';
            arrow.style.borderLeft = '10px solid #95a5a6';
            arrow.style.zIndex = '2';
            
            document.getElementById('flow-view').appendChild(arrow);
        }
        
        function getLevelTitle(level) {
            const titles = {
                1: 'Main Sections (# level)',
                2: 'Sub Sections (## level)',
                3: 'Components (### level)',
                4: 'Functions (#### level)',
                5: 'Details (##### level)',
                6: 'Deep Details (###### level)',
                7: 'Extra Deep (####### level)',
                8: 'Deepest (######## level)'
            };
            return titles[level] || `Level ${level}`;
        }
        
        function jumpToMemo(title) {
            // Switch back to memo mode
            switchToMemoMode();
            
            // Find the target memo container
            const memoTitles = document.querySelectorAll('#memo-view .memo-title');
            let targetContainer = null;
            
            for (let titleEl of memoTitles) {
                if (titleEl.textContent.trim() === title) {
                    targetContainer = titleEl.closest('.memo-container');
                    break;
                }
            }
            
            if (targetContainer) {
                // Recursively expand all parent containers
                expandToTarget(targetContainer);
                
                // Scroll to and highlight the target
                setTimeout(() => {
                    targetContainer.scrollIntoView({ 
                        behavior: 'smooth', 
                        block: 'center' 
                    });
                    
                    // Highlight effect
                    targetContainer.style.background = '#fff3cd';
                    targetContainer.style.borderColor = '#ffc107';
                    setTimeout(() => {
                        targetContainer.style.background = '';
                        targetContainer.style.borderColor = '';
                    }, 2000);
                }, 100);
            }
        }
        
        function expandToTarget(targetContainer) {
            // Find all parent containers by traversing up the DOM
            const parentsToExpand = [];
            let current = targetContainer.parentElement;
            
            while (current && current.id !== 'memo-view') {
                // If this is a children-container, we need to expand it
                if (current.classList.contains('children-container')) {
                    parentsToExpand.push(current);
                }
                current = current.parentElement;
            }
            
            // Expand all parent containers from top to bottom
            parentsToExpand.reverse().forEach(container => {
                if (container.classList.contains('collapsed')) {
                    container.classList.remove('collapsed');
                    container.classList.add('expanded');
                    
                    // Also update the expand icon
                    const header = container.previousElementSibling;
                    if (header && header.classList.contains('memo-header')) {
                        const icon = header.querySelector('.expand-icon');
                        if (icon) {
                            icon.classList.add('expanded');
                        }
                    }
                }
            });
        }
        
        // Zoom functionality
        let currentZoom = 1.0;
        let panX = 0;
        let panY = 0;
        let isPanning = false;
        let lastMouseX = 0;
        let lastMouseY = 0;
        let currentMouseX = 0;
        let currentMouseY = 0;
        
        function updateZoomDisplay() {
            document.getElementById('zoom-level').textContent = Math.round(currentZoom * 100) + '%';
        }
        
        function applyZoom() {
            const container = document.getElementById('zoom-container');
            container.style.transformOrigin = '0 0';
            container.style.transform = `translate(${panX}px, ${panY}px) scale(${currentZoom})`;
            updateZoomDisplay();
        }
        
        function zoomIn() {
            zoomAtPoint(1.25);
        }
        
        function zoomOut() {
            zoomAtPoint(0.8);
        }
        
        function zoomAtPoint(factor) {
            const newZoom = Math.max(Math.min(currentZoom * factor, 5.0), 0.1);
            
            // Use current tracked mouse position or center of screen
            const mouseX = currentMouseX || window.innerWidth / 2;
            const mouseY = currentMouseY || window.innerHeight / 2;
            
            // Calculate the point under the mouse in the original coordinate system
            const pointX = (mouseX - panX) / currentZoom;
            const pointY = (mouseY - panY) / currentZoom;
            
            // Update zoom
            currentZoom = newZoom;
            
            // Adjust pan so the point under the mouse stays in the same place
            panX = mouseX - pointX * currentZoom;
            panY = mouseY - pointY * currentZoom;
            
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
        
        // Track right mouse button state
        let isRightMouseDown = false;
        
        // Mouse wheel zoom (Ctrl+scroll) and pan (right-click+scroll)
        document.addEventListener('wheel', function(e) {
            if (e.ctrlKey || e.metaKey) {
                // Zoom mode
                e.preventDefault();
                
                const delta = e.deltaY > 0 ? 0.9 : 1.1;
                const newZoom = Math.max(Math.min(currentZoom * delta, 5.0), 0.1);
                
                // Get mouse position relative to viewport
                const mouseX = e.clientX;
                const mouseY = e.clientY;
                
                // Calculate the point under the mouse in the original coordinate system
                const pointX = (mouseX - panX) / currentZoom;
                const pointY = (mouseY - panY) / currentZoom;
                
                // Update zoom
                currentZoom = newZoom;
                
                // Adjust pan so the point under the mouse stays in the same place
                panX = mouseX - pointX * currentZoom;
                panY = mouseY - pointY * currentZoom;
                
                applyZoom();
            } else if (isRightMouseDown) {
                // Pan mode with right mouse + scroll
                e.preventDefault();
                
                const panSpeed = 2.0;
                panX -= e.deltaX * panSpeed;
                panY -= e.deltaY * panSpeed;
                
                applyZoom();
            }
            // If no modifier keys, allow normal scrolling
        });
        
        // Track right mouse button for panning
        document.addEventListener('mousedown', function(e) {
            if (e.button === 2) { // Right mouse button
                isRightMouseDown = true;
                e.preventDefault();
            }
        });
        
        document.addEventListener('mouseup', function(e) {
            if (e.button === 2) { // Right mouse button
                isRightMouseDown = false;
            }
        });
        
        // Prevent context menu when right-clicking for panning
        document.addEventListener('contextmenu', function(e) {
            if (isRightMouseDown) {
                e.preventDefault();
            }
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
            // Always track mouse position for zoom centering
            currentMouseX = e.clientX;
            currentMouseY = e.clientY;
            
            if (isPanning) {
                const deltaX = e.clientX - lastMouseX;
                const deltaY = e.clientY - lastMouseY;
                
                panX += deltaX;
                panY += deltaY;
                
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
        html.push_str(if is_horizontal {
            "horizontal-layout"
        } else {
            ""
        });
        html.push_str("\">");

        html.push_str(
            r#"
<div id="zoom-controls">
    <button class="zoom-btn" onclick="zoomOut()">−</button>
    <span id="zoom-level">100%</span>
    <button class="zoom-btn" onclick="zoomIn()">+</button>
    <button class="zoom-btn" onclick="resetZoom()">Reset</button>
    <button class="zoom-btn" onclick="fitToScreen()">Fit</button>
    <div class="control-separator"></div>
    <button id="memo-mode-btn" class="mode-btn active" onclick="switchToMemoMode()">Memo</button>
    <button id="flow-mode-btn" class="mode-btn" onclick="switchToFlowMode()">Flow</button>
</div>
<div id="zoom-container">
    <div id="memo-view" class="view-mode active">
"#,
        );

        if is_horizontal {
            Self::generate_horizontal_layout(&mut html, memos);
        } else {
            for memo in memos {
                Self::generate_memo(&mut html, memo);
            }
        }

        html.push_str("    </div>\n    <div id=\"flow-view\" class=\"view-mode\"></div>\n</div></body>\n</html>");
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
                    html.push_str(&format!(
                        "<div class=\"code-language\">{}</div>",
                        Self::escape_html(&code_block.language)
                    ));
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
                html.push_str(&format!(
                    "{}<li>{}</li>\n",
                    indent,
                    Self::escape_html(item_text)
                ));
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
                .required(true),
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
            Arg::new("layout")
                .short('l')
                .long("layout")
                .value_name("LAYOUT")
                .help("Layout mode: vertical (default) or horizontal")
                .default_value("vertical")
                .value_parser(["vertical", "horizontal"]),
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").unwrap();
    let port: u16 = matches
        .get_one::<String>("port")
        .unwrap()
        .parse()
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

    let websocket_clients: Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<warp::ws::Message>>>> =
        Arc::new(Mutex::new(Vec::new()));
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
                        let memo_content = if is_horizontal_clone {
                            let mut content = String::new();
                            content.push_str("<div class=\"siblings-container\">");
                            for memo in &memos {
                                HtmlGenerator::generate_memo_horizontal(&mut content, memo);
                            }
                            content.push_str("</div>");
                            content
                        } else {
                            memos
                                .iter()
                                .map(|memo| {
                                    let mut memo_html = String::new();
                                    HtmlGenerator::generate_memo(&mut memo_html, memo);
                                    memo_html
                                })
                                .collect::<Vec<String>>()
                                .join("")
                        };

                        let update_msg = serde_json::json!({
                            "type": "update",
                            "html": format!("<body><div id=\"memo-view\" class=\"view-mode active\">{}</div></body>", memo_content)
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
        warp::get().and(warp::path::end()).map(move || {
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

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;

    Ok(())
}
