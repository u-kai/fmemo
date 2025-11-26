import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { App } from './App';

// Mock the hooks
vi.mock('../../hooks/useFileSystem', () => ({
  useFileSystem: vi.fn(() => ({
    directoryStructure: {
      path: '/test',
      items: [
        {
          name: 'test.md',
          path: '/test/test.md',
          type: 'file',
          extension: '.md'
        },
        {
          name: 'src',
          path: '/test/src',
          type: 'directory',
          children: [
            {
              name: 'main.rs',
              path: '/test/src/main.rs',
              type: 'file',
              extension: '.rs'
            }
          ]
        }
      ]
    },
    loading: false,
    error: null,
    refreshDirectory: vi.fn()
  }))
}));

vi.mock('../../hooks/useApi', () => ({
  useApi: vi.fn(() => ({
    loading: false,
    error: null,
    fetchFileContent: vi.fn().mockResolvedValue([
      {
        level: 1,
        title: 'Test Memo',
        content: 'Test content',
        codeBlocks: [],
        children: []
      }
    ]),
    clearError: vi.fn()
  }))
}));

vi.mock('../../hooks/useZoom', () => ({
  useZoom: vi.fn(() => ({
    zoomState: { zoom: 1, panX: 0, panY: 0 },
    zoomIn: vi.fn(),
    zoomOut: vi.fn(),
    resetZoom: vi.fn(),
    fitToScreen: vi.fn()
  }))
}));

vi.mock('../../hooks/useWebSocket', () => ({
  useWebSocket: vi.fn(() => ({
    isConnected: false,
    lastMessage: null,
    error: null,
    sendMessage: vi.fn(),
    reconnect: vi.fn(),
    disconnect: vi.fn()
  }))
}));

describe('App Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders without crashing', () => {
    render(<App />);
    expect(screen.getByText('📁 /test')).toBeInTheDocument();
  });

  it('shows zoom controls', () => {
    render(<App />);
    expect(screen.getByText('−')).toBeInTheDocument();
    expect(screen.getByText('+')).toBeInTheDocument();
    expect(screen.getByText('Reset')).toBeInTheDocument();
    expect(screen.getByText('Fit')).toBeInTheDocument();
    expect(screen.getByText('Memo')).toBeInTheDocument();
    expect(screen.getByText('Flow')).toBeInTheDocument();
  });

  it('can switch between memo and flow modes', async () => {
    render(<App />);
    
    const memoButton = screen.getByText('Memo');
    const flowButton = screen.getByText('Flow');
    
    expect(memoButton).toHaveClass('active');
    expect(flowButton).not.toHaveClass('active');
    
    fireEvent.click(flowButton);
    
    await waitFor(() => {
      expect(flowButton).toHaveClass('active');
      expect(memoButton).not.toHaveClass('active');
    });
  });

  it('shows file explorer with files', () => {
    render(<App />);
    
    expect(screen.getByText('📝')).toBeInTheDocument(); // .md file icon
    expect(screen.getByText('test.md')).toBeInTheDocument();
    expect(screen.getByText('📁')).toBeInTheDocument(); // directory icon
    expect(screen.getByText('src')).toBeInTheDocument();
  });

  it('can select files from explorer', async () => {
    render(<App />);
    
    const testFile = screen.getByText('test.md');
    fireEvent.click(testFile);
    
    await waitFor(() => {
      // The file should be selected and content loaded
      expect(testFile.closest('.file-tree-item-content')).toHaveClass('selected');
    });
  });

  it('handles file selection and loads memo content', async () => {
    const { useApi } = await import('../../hooks/useApi');
    const mockFetchFileContent = vi.fn().mockResolvedValue([
      {
        level: 1,
        title: 'Selected File Content',
        content: 'Content for the selected file',
        codeBlocks: [],
        children: []
      }
    ]);

    useApi.mockReturnValue({
      loading: false,
      error: null,
      fetchFileContent: mockFetchFileContent,
      clearError: vi.fn()
    });

    render(<App />);
    
    const testFile = screen.getByText('test.md');
    fireEvent.click(testFile);
    
    await waitFor(() => {
      expect(mockFetchFileContent).toHaveBeenCalledWith('/test/test.md');
    });
  });
});