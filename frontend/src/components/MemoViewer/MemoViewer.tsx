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