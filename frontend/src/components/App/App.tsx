import React, { useState, useEffect, useCallback, useRef } from "react";
import { FileExplorer } from "../FileExplorer/FileExplorer";
import { MemoViewer } from "../MemoViewer/MemoViewer";
import { FlowView } from "../FlowView/FlowView";
import { ZoomControls } from "../ZoomControls/ZoomControls";
import { useFileSystem } from "../../hooks/useFileSystem";
import { useZoom } from "../../hooks/useZoom";
import { useApi } from "../../hooks/useApi";
import { useWebSocket } from "../../hooks/useWebSocket";
import type { ViewMode, FunctionMemo, FileItem } from "../../types";
import "./App.css";

export const App: React.FC = () => {
  const [selectedFile, setSelectedFile] = useState<string>("");
  const [viewMode, setViewMode] = useState<ViewMode>({
    mode: "memo",
    layout: "vertical",
  });
  
  // Debug viewMode changes
  useEffect(() => {
    console.log('[App] ViewMode changed to:', viewMode);
  }, [viewMode]);
  const [memos, setMemos] = useState<FunctionMemo[]>([]);
  const [controlsCollapsed, setControlsCollapsed] = useState<boolean>(false);

  const { directoryStructure, loading, refreshDirectory } = useFileSystem();
  const { fetchFileContent, refreshFileContent, invalidateCache } = useApi();
  const { isConnected, lastMessage, error: wsError, sendMessage } = useWebSocket();
  // Simple polling for real-time updates
  const [lastFileHash, setLastFileHash] = useState<string>("");

  // Debug WebSocket state changes
  useEffect(() => {
    console.log('[App] WebSocket state changed:', { isConnected, wsError });
  }, [isConnected, wsError]);

  useEffect(() => {
    if (lastMessage) {
      console.log('[App] New WebSocket message:', lastMessage);
    }
  }, [lastMessage]);
  
  // Debug logging removed to prevent infinite re-renders
  const { zoomState, zoomIn, zoomOut, resetZoom, fitToScreen, zoomAtPoint } = useZoom();

  // Mouse wheel zoom (Ctrl+scroll) and keyboard shortcuts
  useEffect(() => {
    const handleKeyboard = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey) {
        console.log('[App] Keyboard shortcut detected:', e.key);
        switch(e.key) {
          case '=':
          case '+':
            e.preventDefault();
            console.log('[App] Zoom in triggered');
            zoomIn();
            break;
          case '-':
            e.preventDefault();
            console.log('[App] Zoom out triggered');
            zoomOut();
            break;
          case '0':
            e.preventDefault();
            console.log('[App] Reset zoom triggered');
            resetZoom();
            break;
        }
      }
    };

    const handleWheel = (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) {
        // Zoom mode
        e.preventDefault();
        console.log('[App] Ctrl+wheel zoom detected:', e.deltaY);
        
        const delta = e.deltaY > 0 ? 0.9 : 1.1;
        const mouseX = e.clientX;
        const mouseY = e.clientY;
        
        zoomAtPoint(delta, mouseX, mouseY);
      }
    };

    window.addEventListener("keydown", handleKeyboard);
    window.addEventListener("wheel", handleWheel, { passive: false });
    
    return () => {
      window.removeEventListener("keydown", handleKeyboard);
      window.removeEventListener("wheel", handleWheel);
    };
  }, [zoomIn, zoomOut, resetZoom, zoomAtPoint]);

  const handleFileSelect = async (filePath: string) => {
    console.log("[App] File selected:", filePath);
    setSelectedFile(filePath);

    // Only fetch content for supported file types
    if (
      filePath.endsWith(".md") ||
      filePath.endsWith(".fmemo") ||
      filePath.endsWith(".rs")
    ) {
      console.log("[App] Fetching content for:", filePath);
      const fetchedMemos = await fetchFileContent(filePath);
      console.log("[App] Fetched memos:", fetchedMemos);

      if (fetchedMemos && fetchedMemos.length > 0) {
        setMemos(fetchedMemos);
        console.log("[App] Memos set to state:", fetchedMemos.length, "items");
      } else {
        console.log("[App] No memos received or empty array, setting empty array");
        setMemos([]);
      }
    } else {
      console.log("[App] File type not supported for memo display");
      setMemos([]);
    }
  };

  const handleDirectorySelect = async (dirPath: string) => {
    console.log("[App] Directory selected:", dirPath);
    setSelectedFile(dirPath);

    if (!directoryStructure) {
      console.log("[App] Directory structure not loaded");
      setMemos([]);
      return;
    }

    // Find the directory in the structure
    const findDirectory = (items: FileItem[], path: string): FileItem | null => {
      for (const item of items) {
        if (item.path === path && item.type === 'directory') {
          return item;
        }
        if (item.children) {
          const found = findDirectory(item.children, path);
          if (found) return found;
        }
      }
      return null;
    };

    const directory = findDirectory(directoryStructure.items, dirPath);
    if (!directory) {
      console.log("[App] Directory not found:", dirPath);
      setMemos([]);
      return;
    }

    // Recursively collect all files
    const collectFiles = (items: FileItem[]): string[] => {
      const files: string[] = [];
      for (const item of items) {
        if (item.type === 'file' && (
          item.path.endsWith(".md") ||
          item.path.endsWith(".fmemo") ||
          item.path.endsWith(".rs")
        )) {
          files.push(item.path);
        }
        if (item.children) {
          files.push(...collectFiles(item.children));
        }
      }
      return files;
    };

    const files = collectFiles(directory.children || []);
    console.log("[App] Files in directory:", files);

    // Fetch all files and combine their memos
    const allMemos: FunctionMemo[] = [];
    for (const filePath of files) {
      console.log("[App] Fetching content for:", filePath);
      const fetchedMemos = await fetchFileContent(filePath);
      if (fetchedMemos && fetchedMemos.length > 0) {
        // Add fetched memos directly without wrapper
        allMemos.push(...fetchedMemos);
      }
    }

    console.log("[App] Combined memos from directory:", allMemos.length, "files");
    setMemos(allMemos);
  };

  const handleModeChange = (mode: "memo" | "flow") => {
    console.log(`[App] Mode change requested: ${mode}`);
    setViewMode((prev) => ({ ...prev, mode }));
  };

  const handleFlowNodeClick = (node: any) => {
    console.log("Flow node clicked:", node);

    // Switch back to memo mode
    setViewMode((prev) => ({ ...prev, mode: "memo" }));

    // Find and scroll to the corresponding memo
    setTimeout(() => {
      jumpToMemo(node.title);
    }, 100);
  };

  const jumpToMemo = (title: string) => {
    // Find the target memo container by title
    const memoTitles = document.querySelectorAll("#memo-view .memo-title");
    let targetContainer: HTMLElement | null = null;

    for (let titleEl of memoTitles) {
      if (titleEl.textContent?.trim() === title) {
        targetContainer = titleEl.closest(".memo-container") as HTMLElement;
        break;
      }
    }

    if (targetContainer) {
      console.log("Found target memo:", title);
      // Recursively expand all parent containers
      expandToTarget(targetContainer);

      // Scroll to and highlight the target
      setTimeout(() => {
        targetContainer!.scrollIntoView({
          behavior: "smooth",
          block: "center",
        });

        // Highlight effect
        const originalBackground = targetContainer!.style.backgroundColor;
        const originalBorderColor = targetContainer!.style.borderColor;

        targetContainer!.style.backgroundColor = "#fff3cd";
        targetContainer!.style.borderColor = "#ffc107";

        setTimeout(() => {
          targetContainer!.style.backgroundColor = originalBackground;
          targetContainer!.style.borderColor = originalBorderColor;
        }, 2000);
      }, 100);
    } else {
      console.warn("Could not find memo with title:", title);
    }
  };

  const expandToTarget = (targetContainer: HTMLElement) => {
    // Find all parent containers by traversing up the DOM
    const parentsToExpand: HTMLElement[] = [];
    let current = targetContainer.parentElement;

    while (current && current.id !== "memo-view") {
      // If this is a children-container, we need to expand it
      if (current.classList.contains("children-container")) {
        parentsToExpand.push(current);
      }
      current = current.parentElement;
    }

    // Expand all parent containers from top to bottom
    parentsToExpand.reverse().forEach((container) => {
      if (container.classList.contains("collapsed")) {
        container.classList.remove("collapsed");
        container.classList.add("expanded");

        // Also update the expand icon
        const header = container.previousElementSibling as HTMLElement;
        if (header && header.classList.contains("memo-header")) {
          const icon = header.querySelector(".expand-icon") as HTMLElement;
          if (icon) {
            icon.classList.add("expanded");
          }
        }
      }
    });
  };

  // Handle WebSocket messages for real-time updates
  const handleWebSocketMessage = useCallback(() => {
    if (!lastMessage) {
      return;
    }
    
    // EMERGENCY FIX: Skip directory_updated messages to prevent infinite loops
    if (lastMessage.type === 'directory_updated') {
      console.log("[App] Skipping directory_updated to prevent loops");
      return;
    }
    
    console.log("[App] Processing WebSocket message:", lastMessage.type);
    console.log("[App] Current selectedFile:", selectedFile);

    switch (lastMessage.type) {
      case "reload":
        // Reload the entire application state
        console.log("Reloading application...");
        refreshDirectory();
        if (selectedFile) {
          // Always reload file content for reload messages
          // Use direct API call instead of handleFileSelect to avoid circular dependency
          refreshFileContent(selectedFile).then((updatedMemos) => {
            if (updatedMemos) {
              setMemos(updatedMemos);
            }
          });
        }
        break;


      case "file_updated":
      case "update":
        console.log("[App] File update detected");
        
        // SIMPLE APPROACH: Always refresh current file content when any file is updated
        if (selectedFile) {
          console.log("[App] Force refreshing current file:", selectedFile);
          refreshFileContent(selectedFile).then((updatedMemos) => {
            if (updatedMemos) {
              console.log("[App] Successfully updated memos:", updatedMemos.length, "items");
              setMemos(updatedMemos);
            }
          });
        }
        break;

      default:
        console.log("Unknown WebSocket message type:", lastMessage.type);
    }
  }, [lastMessage, selectedFile, refreshDirectory, refreshFileContent, invalidateCache]);

  // Simple WebSocket message processing - no duplicate checking
  useEffect(() => {
    console.log("[App] useEffect triggered, lastMessage:", lastMessage?.type || 'null');
    handleWebSocketMessage();
  }, [handleWebSocketMessage]);

  // SMART POLLING: Only update UI when content actually changes
  useEffect(() => {
    if (!selectedFile) return;

    const interval = setInterval(() => {
      console.log("[App] Polling for changes:", selectedFile);
      refreshFileContent(selectedFile).then((updatedMemos) => {
        if (updatedMemos && updatedMemos.length > 0) {
          // Create a hash of the content to detect changes
          const contentHash = JSON.stringify(updatedMemos.map(m => m.title + m.content));
          
          if (contentHash !== lastFileHash) {
            console.log("[App] Content changed, updating UI");
            setMemos(updatedMemos);
            setLastFileHash(contentHash);
          } else {
            console.log("[App] No changes detected, skipping UI update");
          }
        }
      });
    }, 2000);

    return () => clearInterval(interval);
  }, [selectedFile, refreshFileContent, lastFileHash]);


  if (loading || !directoryStructure) {
    return (
      <div className="app-loading">
        <div className="loading-spinner">Loading...</div>
      </div>
    );
  }

  return (
    <div className="app">
      <div className="app-sidebar">
        <FileExplorer
          directoryStructure={directoryStructure}
          onFileSelect={handleFileSelect}
          onDirectorySelect={handleDirectorySelect}
          selectedFile={selectedFile}
        />
      </div>

      <div className="app-main">
        <ZoomControls
          zoomState={zoomState}
          viewMode={viewMode}
          onZoomIn={zoomIn}
          onZoomOut={zoomOut}
          onResetZoom={resetZoom}
          onFitToScreen={fitToScreen}
          onModeChange={handleModeChange}
          collapsed={controlsCollapsed}
          onToggleCollapsed={() => setControlsCollapsed(!controlsCollapsed)}
        />

        {/* WebSocket connection status */}
        <div className="websocket-status">
          <span
            className={`connection-indicator ${isConnected ? "connected" : "disconnected"}`}
          >
            {isConnected ? "🟢" : "🔴"}
          </span>
          <span className="connection-text">
            {isConnected ? "Live updates" : "Disconnected"}
          </span>
          {wsError && (
            <span className="connection-error">Error: {wsError}</span>
          )}
        </div>

        <div
          id="zoom-container"
          className="zoom-container"
          style={{
            transform: `translate(${zoomState.panX}px, ${zoomState.panY}px) scale(${zoomState.zoom})`,
            transformOrigin: "0 0",
            transition: "transform 0.2s ease-out",
            width: "max-content",
            minHeight: "max-content",
          }}
        >
          <MemoViewer memos={memos} viewMode={viewMode} />

          {viewMode.mode === "flow" && (
            <FlowView
              memos={memos}
              className="view-mode active"
              onNodeClick={handleFlowNodeClick}
              key={`flow-${viewMode.mode}-${memos.length}`}
            />
          )}
        </div>
      </div>
    </div>
  );
};
