import React, { useState, useEffect } from 'react';
import { FileExplorer } from '../FileExplorer/FileExplorer';
import { MemoViewer } from '../MemoViewer/MemoViewer';
import { FlowView } from '../FlowView/FlowView';
import { ZoomControls } from '../ZoomControls/ZoomControls';
import { useFileSystem } from '../../hooks/useFileSystem';
import { useZoom } from '../../hooks/useZoom';
import { useApi } from '../../hooks/useApi';
import { useWebSocket } from '../../hooks/useWebSocket';
import type { ViewMode, FunctionMemo } from '../../types';
import './App.css';

export const App: React.FC = () => {
  const [selectedFile, setSelectedFile] = useState<string>('');
  const [viewMode, setViewMode] = useState<ViewMode>({ mode: 'memo', layout: 'vertical' });
  const [memos, setMemos] = useState<FunctionMemo[]>([]);
  
  const { directoryStructure, loading, refreshDirectory } = useFileSystem();
  const { fetchFileContent, refreshFileContent, invalidateCache } = useApi();
  const { isConnected, lastMessage, error: wsError } = useWebSocket();
  const { 
    zoomState, 
    zoomIn, 
    zoomOut, 
    resetZoom, 
    fitToScreen 
  } = useZoom();

  const handleFileSelect = async (filePath: string) => {
    console.log('File selected:', filePath);
    setSelectedFile(filePath);
    
    // Only fetch content for supported file types
    if (filePath.endsWith('.md') || filePath.endsWith('.fmemo') || filePath.endsWith('.rs')) {
      console.log('Fetching content for:', filePath);
      const fetchedMemos = await fetchFileContent(filePath);
      console.log('Fetched memos:', fetchedMemos);
      
      if (fetchedMemos) {
        setMemos(fetchedMemos);
        console.log('Memos set to state:', fetchedMemos.length, 'items');
      } else {
        console.log('No memos received, setting empty array');
        setMemos([]);
      }
    } else {
      console.log('File type not supported for memo display');
      setMemos([]);
    }
  };

  const handleModeChange = (mode: 'memo' | 'flow') => {
    setViewMode(prev => ({ ...prev, mode }));
  };

  // Handle WebSocket messages for real-time updates
  useEffect(() => {
    if (!lastMessage) return;

    console.log('Processing WebSocket message:', lastMessage);

    switch (lastMessage.type) {
      case 'reload':
        // Reload the entire application state
        console.log('Reloading application...');
        refreshDirectory();
        if (selectedFile) {
          handleFileSelect(selectedFile);
        }
        break;

      case 'file_updated':
      case 'update':
        const updatedPath = lastMessage.path;
        if (updatedPath) {
          console.log(`File updated: ${updatedPath}`);
          
          // If the currently selected file was updated, refresh its content
          if (selectedFile === updatedPath || selectedFile.endsWith(updatedPath)) {
            console.log('Refreshing currently selected file');
            refreshFileContent(selectedFile).then(updatedMemos => {
              if (updatedMemos) {
                setMemos(updatedMemos);
              }
            });
          } else {
            // Invalidate cache for the updated file
            invalidateCache(updatedPath);
          }

          // If memos are provided in the message, use them directly
          if (lastMessage.memos && (selectedFile === updatedPath || selectedFile.endsWith(updatedPath))) {
            setMemos(lastMessage.memos);
          }
        }
        break;

      default:
        console.log('Unknown WebSocket message type:', lastMessage.type);
    }
  }, [lastMessage, selectedFile, refreshDirectory, refreshFileContent, invalidateCache]);

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
        />
        
        {/* WebSocket connection status */}
        <div className="websocket-status">
          <span className={`connection-indicator ${isConnected ? 'connected' : 'disconnected'}`}>
            {isConnected ? '🟢' : '🔴'}
          </span>
          <span className="connection-text">
            {isConnected ? 'Live updates' : 'Disconnected'}
          </span>
          {wsError && (
            <span className="connection-error">
              Error: {wsError}
            </span>
          )}
        </div>
        
        <div 
          id="zoom-container"
          className="zoom-container"
          style={{
            transform: `translate(${zoomState.panX}px, ${zoomState.panY}px) scale(${zoomState.zoom})`,
            transformOrigin: '0 0',
            transition: 'transform 0.2s ease-out'
          }}
        >
          <MemoViewer
            memos={memos}
            viewMode={viewMode}
          />
          
          {viewMode.mode === 'flow' && (
            <FlowView
              memos={memos}
              className="view-mode active"
              onNodeClick={(node) => {
                console.log('Flow node clicked:', node);
                // Here you could jump to the corresponding memo
              }}
            />
          )}
        </div>
      </div>
    </div>
  );
};