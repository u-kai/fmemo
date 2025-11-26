import { useState, useCallback } from 'react';
import { apiClient } from '../api/client';
import { useMemoCache } from './useMemoCache';
import type { FunctionMemo, DirectoryStructure } from '../types';

export const useApi = () => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { getCachedMemos, setCachedMemos, invalidateCache } = useMemoCache();

  const fetchDirectoryTree = useCallback(async (path: string = ''): Promise<DirectoryStructure | null> => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await apiClient.getDirectoryTree(path);
      
      if (response.error) {
        setError(response.error);
        return null;
      }
      
      return apiClient.convertToDirectoryStructure(response.data, path || '/Users/kai/refactor-fmemo');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchFileContent = useCallback(async (filePath: string, forceRefresh: boolean = false): Promise<FunctionMemo[] | null> => {
    console.log(`[useApi] fetchFileContent called with: ${filePath}, forceRefresh: ${forceRefresh}`);
    
    // Check cache first unless force refresh is requested
    if (!forceRefresh) {
      const cachedMemos = getCachedMemos(filePath);
      if (cachedMemos) {
        console.log(`[useApi] Using cached memos for ${filePath}:`, cachedMemos.length, 'items');
        return cachedMemos;
      }
    }

    setLoading(true);
    setError(null);
    console.log(`[useApi] Making API call for: ${filePath}`);
    
    try {
      const response = await apiClient.getFileContent(filePath);
      console.log(`[useApi] API response:`, response);
      
      if (response.error) {
        console.error(`[useApi] API returned error:`, response.error);
        setError(response.error);
        return null;
      }
      
      const memos = response.data.memos;
      console.log(`[useApi] Extracted memos:`, memos);
      
      // Cache the result
      setCachedMemos(filePath, memos);
      console.log(`[useApi] Cached ${memos.length} memos for ${filePath}`);
      
      return memos;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      console.error(`[useApi] Error fetching file content:`, err);
      setError(errorMessage);
      return null;
    } finally {
      setLoading(false);
    }
  }, [getCachedMemos, setCachedMemos]);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const refreshFileContent = useCallback((filePath: string) => {
    invalidateCache(filePath);
    return fetchFileContent(filePath, true);
  }, [fetchFileContent, invalidateCache]);

  return {
    loading,
    error,
    fetchDirectoryTree,
    fetchFileContent,
    refreshFileContent,
    clearError,
    invalidateCache
  };
};