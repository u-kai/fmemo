import { useState, useEffect, useCallback } from 'react';
import { apiClient, DirectoryTree, FileContent, WebSocketMessage } from '../api/client';

export function useDirectoryTree() {
  const [tree, setTree] = useState<DirectoryTree | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchTree = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await apiClient.getDirectoryTree();
      setTree(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch directory tree');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTree();
  }, [fetchTree]);

  return { tree, loading, error, refetch: fetchTree };
}

export function useFileContent(filename?: string) {
  const [content, setContent] = useState<FileContent | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchContent = useCallback(async (file: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await apiClient.getFileContent(file);
      setContent(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch file content');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (filename) {
      fetchContent(filename);
    }
  }, [filename, fetchContent]);

  return { content, loading, error, fetchContent };
}

export function useWebSocket() {
  const [connected, setConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);

  useEffect(() => {
    const handleMessage = (message: WebSocketMessage) => {
      setLastMessage(message);
    };

    apiClient.connectWebSocket(handleMessage);

    const checkConnection = setInterval(() => {
      setConnected(apiClient.isWebSocketConnected());
    }, 1000);

    return () => {
      clearInterval(checkConnection);
      apiClient.disconnectWebSocket();
    };
  }, []);

  const addListener = useCallback((listener: (message: WebSocketMessage) => void) => {
    apiClient.addWebSocketListener(listener);
  }, []);

  const removeListener = useCallback((listener: (message: WebSocketMessage) => void) => {
    apiClient.removeWebSocketListener(listener);
  }, []);

  return { connected, lastMessage, addListener, removeListener };
}