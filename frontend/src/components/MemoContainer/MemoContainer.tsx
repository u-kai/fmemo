import { useState } from 'react';
import React from 'react';
import type { FunctionMemo } from '../../types';
import { CodeBlock } from '../CodeBlock/CodeBlock';
import './MemoContainer.css';

interface MemoContainerProps {
  memo: FunctionMemo;
  isHorizontal?: boolean;
  onToggle?: (expanded: boolean) => void;
}

export const MemoContainer: React.FC<MemoContainerProps> = ({ 
  memo, 
  isHorizontal = false,
  onToggle 
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  
  const handleToggle = () => {
    const newExpanded = !isExpanded;
    setIsExpanded(newExpanded);
    onToggle?.(newExpanded);
  };

  const levelClass = `level-${Math.min(memo.level, 8)}`;
  const hasChildren = memo.children.length > 0;
  const noChildrenClass = hasChildren ? '' : ' no-children';

  const renderContent = () => {
    if (!memo.content.trim()) return null;
    
    return (
      <div className="memo-content" 
           dangerouslySetInnerHTML={{ __html: markdownToHtml(memo.content) }} 
      />
    );
  };

  const renderCodeBlocks = () => {
    if (memo.codeBlocks.length === 0) return null;
    
    return (
      <div className="memo-body">
        {memo.codeBlocks.map((codeBlock, index) => (
          <CodeBlock key={index} codeBlock={codeBlock} />
        ))}
      </div>
    );
  };

  const renderChildren = () => {
    if (!hasChildren) return null;

    return (
      <div className={`children-container ${isExpanded ? 'expanded' : 'collapsed'}`}>
        {memo.children.length > 1 && isHorizontal ? (
          <div className="siblings-container">
            {memo.children.map((child, index) => (
              <MemoContainer key={index} memo={child} isHorizontal={isHorizontal} />
            ))}
          </div>
        ) : (
          memo.children.map((child, index) => (
            <MemoContainer key={index} memo={child} isHorizontal={isHorizontal} />
          ))
        )}
      </div>
    );
  };

  return (
    <div className={`memo-container ${levelClass}${noChildrenClass}`}>
      <div className="memo-header" onClick={handleToggle}>
        <div className="memo-title-container">
          <span className={`expand-icon ${isExpanded ? 'expanded' : ''}`}>▶</span>
          {React.createElement(
            `h${Math.min(memo.level, 6)}`, 
            { className: 'memo-title' }, 
            memo.title
          )}
        </div>
      </div>
      
      {renderContent()}
      {renderCodeBlocks()}
      {renderChildren()}
    </div>
  );
};

function markdownToHtml(text: string): string {
  let html = '';
  let listLevels: number[] = [];
  let currentParagraph = '';

  for (const line of text.split('\n')) {
    const leadingSpaces = line.length - line.trimStart().length;
    const trimmed = line.trim();

    if (trimmed.startsWith('- ')) {
      if (currentParagraph.trim()) {
        html += `<p>${currentParagraph.trim()}</p>\n`;
        currentParagraph = '';
      }

      const currentLevel = leadingSpaces;
      
      while (listLevels.length > 0 && listLevels[listLevels.length - 1] >= currentLevel) {
        listLevels.pop();
        html += '</ul>\n';
      }

      if (listLevels.length === 0 || listLevels[listLevels.length - 1] < currentLevel) {
        listLevels.push(currentLevel);
        html += '<ul>\n';
      }

      const itemText = trimmed.slice(2);
      const indent = '  '.repeat(listLevels.length);
      html += `${indent}<li>${escapeHtml(itemText)}</li>\n`;
    } else if (trimmed === '') {
      while (listLevels.length > 0) {
        listLevels.pop();
        html += '</ul>\n';
      }
      if (currentParagraph.trim()) {
        html += `<p>${currentParagraph.trim()}</p>\n`;
        currentParagraph = '';
      }
    } else {
      while (listLevels.length > 0) {
        listLevels.pop();
        html += '</ul>\n';
      }
      if (currentParagraph) {
        currentParagraph += ' ';
      }
      currentParagraph += trimmed;
    }
  }

  while (listLevels.length > 0) {
    listLevels.pop();
    html += '</ul>\n';
  }
  if (currentParagraph.trim()) {
    html += `<p>${currentParagraph.trim()}</p>\n`;
  }

  return html;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;');
}