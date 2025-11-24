import React from 'react';
import type { FunctionMemo, ViewMode } from '../../types';
import { MemoContainer } from '../MemoContainer/MemoContainer';
import './MemoViewer.css';

interface MemoViewerProps {
  memos: FunctionMemo[];
  viewMode: ViewMode;
  className?: string;
}

export const MemoViewer: React.FC<MemoViewerProps> = ({ 
  memos, 
  viewMode,
  className = '' 
}) => {
  const isHorizontal = viewMode.layout === 'horizontal';
  const isActive = viewMode.mode === 'memo';
  
  console.log('[MemoViewer] Rendering with:', { 
    memosCount: memos.length, 
    viewMode, 
    isActive, 
    memos: memos.map(m => ({ title: m.title, level: m.level }))
  });

  if (!isActive) {
    console.log('[MemoViewer] Not active, returning empty div');
    return <div className={`view-mode ${className}`}></div>;
  }

  if (memos.length === 0) {
    console.log('[MemoViewer] No memos to display');
    return (
      <div 
        id="memo-view" 
        className={`view-mode active ${className}`}
      >
        <div style={{ padding: '20px', color: '#666', textAlign: 'center' }}>
          No memo content available for this file.
        </div>
      </div>
    );
  }
  
  if (isHorizontal) {
    return (
      <div 
        id="memo-view" 
        className={`view-mode horizontal-layout ${isActive ? 'active' : ''} ${className}`}
      >
        <div className="siblings-container">
          {memos.map((memo, index) => (
            <MemoContainer key={index} memo={memo} isHorizontal={true} />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div 
      id="memo-view" 
      className={`view-mode ${isActive ? 'active' : ''} ${className}`}
    >
      {memos.map((memo, index) => (
        <MemoContainer key={index} memo={memo} isHorizontal={false} />
      ))}
    </div>
  );
};