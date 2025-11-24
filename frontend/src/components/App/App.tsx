import React, { useState } from 'react';
import { FileExplorer } from '../FileExplorer/FileExplorer';
import { MemoViewer } from '../MemoViewer/MemoViewer';
import { FlowView } from '../FlowView/FlowView';
import { ZoomControls } from '../ZoomControls/ZoomControls';
import { useFileSystem } from '../../hooks/useFileSystem';
import { useZoom } from '../../hooks/useZoom';
import type { ViewMode, FunctionMemo } from '../../types';
import './App.css';

export const App: React.FC = () => {
  const [selectedFile, setSelectedFile] = useState<string>('');
  const [viewMode, setViewMode] = useState<ViewMode>({ mode: 'memo', layout: 'vertical' });
  const [memos, setMemos] = useState<FunctionMemo[]>([]);
  
  const { directoryStructure, loading } = useFileSystem();
  const { 
    zoomState, 
    zoomIn, 
    zoomOut, 
    resetZoom, 
    fitToScreen 
  } = useZoom();

  const handleFileSelect = async (filePath: string) => {
    setSelectedFile(filePath);
    
    // Mock memo data for demonstration
    if (filePath.endsWith('.md') || filePath.endsWith('.rs')) {
      const mockMemos: FunctionMemo[] = [
        {
          level: 1,
          title: `Content from ${filePath.split('/').pop()}`,
          content: `This is the content from the selected file: ${filePath}
          
You can add more details here:
- File analysis
- Structure overview  
- Important notes`,
          codeBlocks: [
            {
              language: filePath.endsWith('.rs') ? 'rust' : 'markdown',
              code: filePath.endsWith('.rs') 
                ? `fn main() {\n    println!("Hello from ${filePath}");\n}`
                : `# Sample content\n\nThis would be the actual file content.`
            }
          ],
          children: [
            {
              level: 2,
              title: 'Sub-section',
              content: 'Additional details about this file.',
              codeBlocks: [],
              children: []
            }
          ]
        }
      ];
      setMemos(mockMemos);
    } else {
      setMemos([]);
    }
  };

  const handleModeChange = (mode: 'memo' | 'flow') => {
    setViewMode(prev => ({ ...prev, mode }));
  };

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