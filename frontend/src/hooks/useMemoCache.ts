import { useState, useCallback } from 'react';
import type { FunctionMemo } from '../types';

interface MemoCache {
  [filePath: string]: {
    memos: FunctionMemo[];
    timestamp: number;
  };
}

export const useMemoCache = () => {
  const [cache, setCache] = useState<MemoCache>({});

  const getCachedMemos = useCallback((filePath: string): FunctionMemo[] | null => {
    const cached = cache[filePath];
    if (!cached) return null;

    // Cache expires after 5 minutes
    const now = Date.now();
    const isExpired = now - cached.timestamp > 5 * 60 * 1000;
    
    if (isExpired) {
      // Remove expired entry
      setCache(prev => {
        const { [filePath]: removed, ...rest } = prev;
        return rest;
      });
      return null;
    }

    return cached.memos;
  }, [cache]);

  const setCachedMemos = useCallback((filePath: string, memos: FunctionMemo[]) => {
    setCache(prev => ({
      ...prev,
      [filePath]: {
        memos,
        timestamp: Date.now()
      }
    }));
  }, []);

  const invalidateCache = useCallback((filePath?: string) => {
    if (filePath) {
      setCache(prev => {
        const { [filePath]: removed, ...rest } = prev;
        return rest;
      });
    } else {
      setCache({});
    }
  }, []);

  const getCacheStats = useCallback(() => {
    const entries = Object.keys(cache).length;
    const totalSize = Object.values(cache).reduce((sum, entry) => 
      sum + JSON.stringify(entry.memos).length, 0);
    
    return {
      entries,
      totalSize,
      files: Object.keys(cache)
    };
  }, [cache]);

  return {
    getCachedMemos,
    setCachedMemos,
    invalidateCache,
    getCacheStats
  };
};