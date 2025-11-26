import React, { useState } from 'react';
import type { FileItem } from '../../types';

interface FileTreeItemProps {
  item: FileItem;
  onSelect: (filePath: string) => void;
  onDirectorySelect?: (dirPath: string) => void;
  selectedPath?: string;
  level: number;
}

export const FileTreeItem: React.FC<FileTreeItemProps> = ({
  item,
  onSelect,
  onDirectorySelect,
  selectedPath,
  level
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  
  const isSelected = selectedPath === item.path;
  const isDirectory = item.type === 'directory';
  const hasChildren = isDirectory && item.children && item.children.length > 0;
  
  const handleClick = () => {
    if (isDirectory) {
      setIsExpanded(!isExpanded);
      if (onDirectorySelect) {
        onDirectorySelect(item.path);
      }
    } else {
      onSelect(item.path);
    }
  };

  const getFileIcon = (fileItem: FileItem) => {
    if (fileItem.type === 'directory') {
      return isExpanded ? '📂' : '📁';
    }
    
    const ext = fileItem.extension?.toLowerCase();
    switch (ext) {
      case '.md':
      case '.markdown':
        return '📝';
      case '.rs':
        return '🦀';
      case '.js':
      case '.jsx':
        return '📄';
      case '.ts':
      case '.tsx':
        return '📘';
      case '.json':
        return '📋';
      case '.css':
        return '🎨';
      case '.html':
        return '🌐';
      default:
        return '📄';
    }
  };

  const indent = level * 16;

  return (
    <div className="file-tree-item">
      <div 
        className={`file-tree-item-content ${isSelected ? 'selected' : ''} ${isDirectory ? 'directory' : 'file'}`}
        onClick={handleClick}
        style={{ paddingLeft: `${indent + 8}px` }}
      >
        {hasChildren && (
          <span className={`expand-arrow ${isExpanded ? 'expanded' : ''}`}>
            ▶
          </span>
        )}
        <span className="file-icon">
          {getFileIcon(item)}
        </span>
        <span className="file-name">
          {item.name}
        </span>
      </div>
      
      {hasChildren && isExpanded && (
        <div className="file-tree-children">
          {item.children!.map((child, index) => (
            <FileTreeItem
              key={`${child.path}-${index}`}
              item={child}
              onSelect={onSelect}
              onDirectorySelect={onDirectorySelect}
              selectedPath={selectedPath}
              level={level + 1}
            />
          ))}
        </div>
      )}
    </div>
  );
};