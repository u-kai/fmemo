import { useState, useEffect } from 'react';
import { useApi } from './useApi';
import type { DirectoryStructure } from '../types';

export const useFileSystem = (rootPath?: string) => {
  const [directoryStructure, setDirectoryStructure] = useState<DirectoryStructure | null>(null);
  const { loading, error, fetchDirectoryTree, clearError } = useApi();

  useEffect(() => {
    const loadDirectory = async () => {
      const structure = await fetchDirectoryTree(rootPath);
      if (structure) {
        setDirectoryStructure(structure);
      }
    };

    loadDirectory();
  }, [rootPath, fetchDirectoryTree]);

  const refreshDirectory = async () => {
    clearError();
    const structure = await fetchDirectoryTree(rootPath);
    if (structure) {
      setDirectoryStructure(structure);
    }
  };

  return {
    directoryStructure,
    loading,
    error,
    refreshDirectory
  };
};