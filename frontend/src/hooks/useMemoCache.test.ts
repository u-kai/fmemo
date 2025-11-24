import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useMemoCache } from './useMemoCache';
import type { FunctionMemo } from '../types';

describe('useMemoCache', () => {
  let mockMemos: FunctionMemo[];

  beforeEach(() => {
    mockMemos = [
      {
        level: 1,
        title: 'Test Memo',
        content: 'Test content',
        codeBlocks: [],
        children: []
      }
    ];
    vi.clearAllTimers();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should cache and retrieve memos', () => {
    const { result } = renderHook(() => useMemoCache());
    
    // Initially no cache
    expect(result.current.getCachedMemos('test.md')).toBeNull();
    
    // Set cache
    act(() => {
      result.current.setCachedMemos('test.md', mockMemos);
    });
    
    // Should retrieve cached memos
    expect(result.current.getCachedMemos('test.md')).toEqual(mockMemos);
  });

  it('should expire cache after 5 minutes', () => {
    const { result } = renderHook(() => useMemoCache());
    
    // Set cache
    act(() => {
      result.current.setCachedMemos('test.md', mockMemos);
    });
    
    // Should have cached data
    expect(result.current.getCachedMemos('test.md')).toEqual(mockMemos);
    
    // Fast forward 6 minutes
    act(() => {
      vi.advanceTimersByTime(6 * 60 * 1000);
    });
    
    // Cache should be expired
    expect(result.current.getCachedMemos('test.md')).toBeNull();
  });

  it('should invalidate specific cache entry', () => {
    const { result } = renderHook(() => useMemoCache());
    
    // Set multiple cache entries
    act(() => {
      result.current.setCachedMemos('test1.md', mockMemos);
      result.current.setCachedMemos('test2.md', mockMemos);
    });
    
    // Both should be cached
    expect(result.current.getCachedMemos('test1.md')).toEqual(mockMemos);
    expect(result.current.getCachedMemos('test2.md')).toEqual(mockMemos);
    
    // Invalidate one
    act(() => {
      result.current.invalidateCache('test1.md');
    });
    
    // Only test1.md should be invalidated
    expect(result.current.getCachedMemos('test1.md')).toBeNull();
    expect(result.current.getCachedMemos('test2.md')).toEqual(mockMemos);
  });

  it('should clear all cache when no path specified', () => {
    const { result } = renderHook(() => useMemoCache());
    
    // Set multiple cache entries
    act(() => {
      result.current.setCachedMemos('test1.md', mockMemos);
      result.current.setCachedMemos('test2.md', mockMemos);
    });
    
    // Clear all cache
    act(() => {
      result.current.invalidateCache();
    });
    
    // All should be cleared
    expect(result.current.getCachedMemos('test1.md')).toBeNull();
    expect(result.current.getCachedMemos('test2.md')).toBeNull();
  });

  it('should provide cache statistics', () => {
    const { result } = renderHook(() => useMemoCache());
    
    // Initially empty
    expect(result.current.getCacheStats().entries).toBe(0);
    
    // Add cache entries
    act(() => {
      result.current.setCachedMemos('test1.md', mockMemos);
      result.current.setCachedMemos('test2.md', mockMemos);
    });
    
    const stats = result.current.getCacheStats();
    expect(stats.entries).toBe(2);
    expect(stats.files).toEqual(['test1.md', 'test2.md']);
    expect(stats.totalSize).toBeGreaterThan(0);
  });
});