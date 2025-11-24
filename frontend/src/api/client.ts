import type { FunctionMemo, DirectoryStructure, FileItem } from '../types';

export interface ApiResponse<T> {
  data: T;
  error?: string;
}

export interface ApiDirectoryTree {
  files: string[];
  directories: string[];
}

export interface ApiFileContent {
  path: string;
  content: string;
  memos: FunctionMemo[];
}

export class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = '') {
    // In development (Vite dev server), use relative URLs that get proxied
    // In production, use the provided baseUrl or default to current origin
    this.baseUrl = import.meta.env.DEV ? '' : baseUrl || window.location.origin;
  }

  async getDirectoryTree(path: string = ''): Promise<ApiResponse<ApiDirectoryTree>> {
    try {
      const url = path ? `${this.baseUrl}/api/files/${encodeURIComponent(path)}` : `${this.baseUrl}/api/root`;
      const response = await fetch(url);
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const data = await response.json();
      return { data };
    } catch (error) {
      return { 
        data: { files: [], directories: [] },
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  async getFileContent(filePath: string): Promise<ApiResponse<ApiFileContent>> {
    try {
      // Extract filename from full path for API call
      // Server expects just the filename, not the full path
      const filename = filePath.includes('/') ? filePath.split('/').pop() : filePath;
      console.log(`[ApiClient] Calling API with filename: ${filename} (from path: ${filePath})`);
      
      const response = await fetch(`${this.baseUrl}/api/file/${encodeURIComponent(filename || filePath)}`);
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const data = await response.json();
      console.log(`[ApiClient] Received response:`, data);
      return { data };
    } catch (error) {
      console.error(`[ApiClient] Error fetching file:`, error);
      return {
        data: { path: filePath, content: '', memos: [] },
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  // Convert API response to frontend types
  convertToDirectoryStructure(
    apiData: ApiDirectoryTree, 
    currentPath: string = '/Users/kai/refactor-fmemo'
  ): DirectoryStructure {
    const items: FileItem[] = [];

    // Add directories
    apiData.directories.forEach(dir => {
      items.push({
        name: dir,
        path: `${currentPath}/${dir}`,
        type: 'directory',
        children: [] // Will be populated when expanded
      });
    });

    // Add files
    apiData.files.forEach(file => {
      const extension = this.getFileExtension(file);
      items.push({
        name: file,
        path: `${currentPath}/${file}`,
        type: 'file',
        extension
      });
    });

    return {
      path: currentPath,
      items: items.sort((a, b) => {
        // Sort directories first, then files
        if (a.type !== b.type) {
          return a.type === 'directory' ? -1 : 1;
        }
        return a.name.localeCompare(b.name);
      })
    };
  }

  private getFileExtension(filename: string): string {
    const lastDot = filename.lastIndexOf('.');
    return lastDot !== -1 ? filename.substring(lastDot) : '';
  }
}

// Mock API client for development
export class MockApiClient extends ApiClient {
  private mockData: Record<string, ApiDirectoryTree> = {
    '': {
      files: ['README.md', 'Cargo.toml', 'input.md', 'test_hierarchy.md'],
      directories: ['src', 'frontend', 'target']
    },
    'src': {
      files: ['main.rs', 'lib.rs', 'parser.rs', 'server.rs'],
      directories: []
    },
    'frontend': {
      files: ['package.json', 'vite.config.ts', 'tsconfig.json'],
      directories: ['src', 'public']
    },
    'frontend/src': {
      files: ['App.tsx', 'main.tsx', 'index.css'],
      directories: ['components', 'hooks', 'types', 'stories']
    }
  };

  private mockFileContent: Record<string, string> = {
    'README.md': `# Function Memo Viewer

A hierarchical markdown memo viewer with real-time updates.

## Features
- Hierarchical structure visualization
- Real-time file watching
- Flow diagram view
- WebSocket updates`,
    'src/main.rs': `use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Function Memo server...");
    Ok(())
}`,
    'input.md': `# Sample Input File

## Main Function
This is a sample function that demonstrates the memo structure.

\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\`

### Helper Functions
Additional helper functions go here.

#### Process Data
Processes input data and returns results.`
  };

  async getDirectoryTree(path: string = ''): Promise<ApiResponse<ApiDirectoryTree>> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 200));

    const data = this.mockData[path] || { files: [], directories: [] };
    return { data };
  }

  async getFileContent(filePath: string): Promise<ApiResponse<ApiFileContent>> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 300));

    const content = this.mockFileContent[filePath] || `# ${filePath}\n\nContent for ${filePath}`;
    
    // Mock parsed memos (in real API, this would come from Rust backend)
    const memos = this.getMockMemos(filePath);

    return {
      data: {
        path: filePath,
        content,
        memos
      }
    };
  }

  private getMockMemos(filePath: string): FunctionMemo[] {
    // Mock data that simulates what the Rust API would return
    if (filePath === 'README.md') {
      return [
        {
          level: 1,
          title: 'Function Memo Viewer',
          content: 'A hierarchical markdown memo viewer with real-time updates.',
          codeBlocks: [],
          children: [
            {
              level: 2,
              title: 'Features',
              content: '- Hierarchical structure visualization\n- Real-time file watching\n- Flow diagram view\n- WebSocket updates',
              codeBlocks: [],
              children: []
            }
          ]
        }
      ];
    } else if (filePath === 'src/main.rs') {
      return [
        {
          level: 1,
          title: 'Main Function',
          content: 'Entry point for the Function Memo server application.',
          codeBlocks: [
            {
              language: 'rust',
              code: 'use clap::{Arg, Command};\n\n#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {\n    println!("Starting Function Memo server...");\n    Ok(())\n}'
            }
          ],
          children: [
            {
              level: 2,
              title: 'Server Setup',
              content: 'Initializes the web server and starts listening for requests.',
              codeBlocks: [],
              children: []
            }
          ]
        }
      ];
    } else if (filePath === 'input.md') {
      return [
        {
          level: 1,
          title: 'Sample Input File',
          content: 'This is a sample file to demonstrate the memo structure.',
          codeBlocks: [],
          children: [
            {
              level: 2,
              title: 'Main Function',
              content: 'This is a sample function that demonstrates the memo structure.',
              codeBlocks: [
                {
                  language: 'rust',
                  code: 'fn main() {\n    println!("Hello, world!");\n}'
                }
              ],
              children: [
                {
                  level: 3,
                  title: 'Helper Functions',
                  content: 'Additional helper functions go here.',
                  codeBlocks: [],
                  children: [
                    {
                      level: 4,
                      title: 'Process Data',
                      content: 'Processes input data and returns results.',
                      codeBlocks: [],
                      children: []
                    }
                  ]
                }
              ]
            }
          ]
        }
      ];
    }

    // Default mock for other files
    return [
      {
        level: 1,
        title: `Content from ${filePath}`,
        content: `This is mock content for the file: ${filePath}`,
        codeBlocks: [],
        children: [
          {
            level: 2,
            title: 'Sub-section',
            content: 'Additional details about this file.',
            codeBlocks: [],
            children: []
          }
        ]
      }
    ];
  }
}

// Export default instance
export const apiClient = new ApiClient();