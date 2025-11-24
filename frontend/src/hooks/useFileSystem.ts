import { useState, useEffect } from 'react';
import type { DirectoryStructure, FileItem } from '../types';

export const useFileSystem = (rootPath?: string) => {
  const [directoryStructure, setDirectoryStructure] = useState<DirectoryStructure | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Mock data for demonstration
  const mockDirectoryStructure: DirectoryStructure = {
    path: '/Users/kai/refactor-fmemo',
    items: [
      {
        name: 'src',
        path: '/Users/kai/refactor-fmemo/src',
        type: 'directory',
        children: [
          {
            name: 'main.rs',
            path: '/Users/kai/refactor-fmemo/src/main.rs',
            type: 'file',
            extension: '.rs'
          },
          {
            name: 'lib.rs',
            path: '/Users/kai/refactor-fmemo/src/lib.rs',
            type: 'file',
            extension: '.rs'
          },
          {
            name: 'parser.rs',
            path: '/Users/kai/refactor-fmemo/src/parser.rs',
            type: 'file',
            extension: '.rs'
          }
        ]
      },
      {
        name: 'frontend',
        path: '/Users/kai/refactor-fmemo/frontend',
        type: 'directory',
        children: [
          {
            name: 'src',
            path: '/Users/kai/refactor-fmemo/frontend/src',
            type: 'directory',
            children: [
              {
                name: 'App.tsx',
                path: '/Users/kai/refactor-fmemo/frontend/src/App.tsx',
                type: 'file',
                extension: '.tsx'
              },
              {
                name: 'components',
                path: '/Users/kai/refactor-fmemo/frontend/src/components',
                type: 'directory',
                children: [
                  {
                    name: 'MemoViewer',
                    path: '/Users/kai/refactor-fmemo/frontend/src/components/MemoViewer',
                    type: 'directory',
                    children: [
                      {
                        name: 'MemoViewer.tsx',
                        path: '/Users/kai/refactor-fmemo/frontend/src/components/MemoViewer/MemoViewer.tsx',
                        type: 'file',
                        extension: '.tsx'
                      }
                    ]
                  }
                ]
              }
            ]
          },
          {
            name: 'package.json',
            path: '/Users/kai/refactor-fmemo/frontend/package.json',
            type: 'file',
            extension: '.json'
          }
        ]
      },
      {
        name: 'README.md',
        path: '/Users/kai/refactor-fmemo/README.md',
        type: 'file',
        extension: '.md'
      },
      {
        name: 'Cargo.toml',
        path: '/Users/kai/refactor-fmemo/Cargo.toml',
        type: 'file',
        extension: '.toml'
      },
      {
        name: 'test_hierarchy.md',
        path: '/Users/kai/refactor-fmemo/test_hierarchy.md',
        type: 'file',
        extension: '.md'
      },
      {
        name: 'input.md',
        path: '/Users/kai/refactor-fmemo/input.md',
        type: 'file',
        extension: '.md'
      }
    ]
  };

  useEffect(() => {
    // In a real implementation, this would fetch from an API
    setLoading(true);
    
    // Simulate API call
    setTimeout(() => {
      setDirectoryStructure(mockDirectoryStructure);
      setLoading(false);
    }, 500);
  }, [rootPath]);

  const refreshDirectory = () => {
    setLoading(true);
    setTimeout(() => {
      setDirectoryStructure(mockDirectoryStructure);
      setLoading(false);
    }, 500);
  };

  return {
    directoryStructure,
    loading,
    error,
    refreshDirectory
  };
};