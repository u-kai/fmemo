import { useState, useEffect, useRef, useCallback } from 'react';

interface WebSocketMessage {
  type: 'reload' | 'update' | 'file_updated' | 'directory_updated';
  path?: string;
  file_path?: string;
  html?: string;
  memos?: any[];
  tree?: any;
}

// Determine sensible default WS URL
const defaultWsUrl = (() => {
  // Allow override via env
  const envUrl = (import.meta as any)?.env?.VITE_WS_URL as string | undefined;
  if (envUrl) return envUrl;
  
  // In dev, use Vite's dev server proxy (which forwards to backend)
  if ((import.meta as any)?.env?.DEV && typeof window !== 'undefined') {
    return `ws://${window.location.host}/ws`;
  }
  // Fallback to direct backend connection
  if ((import.meta as any)?.env?.DEV) return 'ws://localhost:8080/ws';
  // In production (served by the same server), use current host
  if (typeof window !== 'undefined') return `ws://${window.location.host}/ws`;
  return 'ws://localhost:8080/ws';
})();

export const useWebSocket = (url: string = defaultWsUrl) => {
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);
  const [error, setError] = useState<string | null>(null);
  
  const ws = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;

  const connect = useCallback(() => {
    try {
      console.log('🔄 Attempting WebSocket connection to:', url);
      console.log('🌐 Current environment:', {
        isDev: (import.meta as any)?.env?.DEV,
        host: typeof window !== 'undefined' ? window.location.host : 'N/A'
      });
      ws.current = new WebSocket(url);
      
      ws.current.onopen = () => {
        console.log('✅ WebSocket connected successfully to:', url);
        setIsConnected(true);
        setError(null);
        reconnectAttempts.current = 0;
      };
      
      ws.current.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          console.log('WebSocket message received:', message);
          setLastMessage(message);
        } catch (err) {
          console.error('Failed to parse WebSocket message:', err);
        }
      };
      
      ws.current.onclose = (event) => {
        console.log('WebSocket disconnected:', event.code, event.reason);
        setIsConnected(false);
        
        // Attempt to reconnect if it wasn't a manual close
        if (event.code !== 1000 && reconnectAttempts.current < maxReconnectAttempts) {
          const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.current), 30000);
          console.log(`Attempting to reconnect in ${delay}ms...`);
          
          reconnectTimeoutRef.current = window.setTimeout(() => {
            reconnectAttempts.current++;
            connect();
          }, delay);
        } else if (reconnectAttempts.current >= maxReconnectAttempts) {
          setError('Failed to reconnect to WebSocket after multiple attempts');
        }
      };
      
      ws.current.onerror = (event) => {
        console.error('WebSocket error:', event);
        setError('WebSocket connection error');
      };
      
    } catch (err) {
      console.error('Failed to create WebSocket connection:', err);
      setError('Failed to create WebSocket connection');
    }
  }, []);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    
    if (ws.current) {
      ws.current.close(1000, 'Manual disconnect');
      ws.current = null;
    }
    
    setIsConnected(false);
  }, []);

  const sendMessage = useCallback((message: any) => {
    if (ws.current && ws.current.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket is not connected');
    }
  }, []);

  useEffect(() => {
    console.log('🚀 WebSocket hook initializing...');
    connect();
    
    return () => {
      console.log('useWebSocket cleanup - disconnecting');
      disconnect();
    };
  }, []); // 空の依存関係配列で初回のみ実行


  return {
    isConnected,
    lastMessage,
    error,
    sendMessage,
    reconnect: connect,
    disconnect
  };
};
