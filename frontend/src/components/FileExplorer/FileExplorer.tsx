import React from 'react';
import type { DirectoryStructure } from '../../types';
import { FileTreeItem } from './FileTreeItem';
import './FileExplorer.css';

interface FileExplorerProps {
  directoryStructure: DirectoryStructure;
  onFileSelect: (filePath: string) => void;
  onDirectorySelect?: (dirPath: string) => void;
  selectedFile?: string;
  className?: string;
}

export const FileExplorer: React.FC<FileExplorerProps> = ({
  directoryStructure,
  onFileSelect,
  onDirectorySelect,
  selectedFile,
  className = ''
}) => {
  return (
    <div className={`file-explorer ${className}`}>
      <div className="file-explorer-header">
        <h3 className="file-explorer-title">
          📁 Files
        </h3>
      </div>
      <div className="file-explorer-content">
        <div className="file-tree">
          {directoryStructure.items.map((item, index) => (
            <FileTreeItem
              key={`${item.path}-${index}`}
              item={item}
              onSelect={onFileSelect}
              onDirectorySelect={onDirectorySelect}
              selectedPath={selectedFile}
              level={0}
            />
          ))}
        </div>
      </div>
    </div>
  );
};
