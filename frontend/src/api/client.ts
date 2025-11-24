import { CodeBlock } from '../types';

export interface DirectoryTree {
  path: string;
  files: string[];
  subdirectories: DirectoryTree[];
}

export interface Memo {
  level: number;
  title: string;
  description?: string;
  content?: string;
  code_blocks: CodeBlock[];
  children: Memo[];
}

export interface FileContent {
  memos: Memo[];
  last_modified?: number;
}

export interface FileChangeNotification {
  type: 'file_updated';
  file_path: string;
  memos: Memo[];
}

export interface DirectoryChangeNotification {
  type: 'directory_updated';
  tree: DirectoryTree;
}

export type WebSocketMessage = FileChangeNotification | DirectoryChangeNotification;

class ApiClient {
  private baseUrl: string;
  private ws: WebSocket | null = null;
  private wsListeners: ((message: WebSocketMessage) => void)[] = [];

  constructor(baseUrl: string = '') {
    this.baseUrl = baseUrl;
  }

  async getDirectoryTree(): Promise<DirectoryTree> {
    const response = await fetch(`${this.baseUrl}/api/root`);
    if (!response.ok) {
      throw new Error(`Failed to fetch directory tree: ${response.statusText}`);
    }
    return response.json();
  }

  async getFileContent(filename: string): Promise<FileContent> {
    const response = await fetch(`${this.baseUrl}/api/files/${encodeURIComponent(filename)}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch file content: ${response.statusText}`);
    }
    return response.json();
  }

  connectWebSocket(onMessage?: (message: WebSocketMessage) => void): void {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    
    this.ws = new WebSocket(wsUrl);
    
    this.ws.onopen = () => {
      console.log('WebSocket connected');
    };
    
    this.ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        if (onMessage) {
          onMessage(message);
        }
        this.wsListeners.forEach(listener => listener(message));
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };
    
    this.ws.onclose = () => {
      console.log('WebSocket disconnected');
      // Auto-reconnect after 3 seconds
      setTimeout(() => {
        if (!this.ws || this.ws.readyState === WebSocket.CLOSED) {
          this.connectWebSocket(onMessage);
        }
      }, 3000);
    };
    
    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }

  addWebSocketListener(listener: (message: WebSocketMessage) => void): void {
    this.wsListeners.push(listener);
  }

  removeWebSocketListener(listener: (message: WebSocketMessage) => void): void {
    this.wsListeners = this.wsListeners.filter(l => l !== listener);
  }

  disconnectWebSocket(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.wsListeners = [];
  }

  isWebSocketConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}

export const apiClient = new ApiClient();