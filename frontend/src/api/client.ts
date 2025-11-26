import type { FunctionMemo, DirectoryStructure, FileItem } from '../types';

export interface ApiResponse<T> {
  data: T;
  error?: string;
}

export interface ApiDirectoryTree {
  path: string;
  files: string[];
  subdirectories: ApiDirectoryTree[];
}

export interface ApiFileContent {
  path: string;
  content: string;
  memos: FunctionMemo[];
}

export class ApiClient {
  private baseUrl: string;
  private lastRootPath: string | null = null;

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
        data: { path: '', files: [], subdirectories: [] },
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  async getFileContent(filePath: string): Promise<ApiResponse<ApiFileContent>> {
    try {
      // Compute path relative to root if possible (server expects path relative to configured root)
      let relativePath = filePath;
      if (this.lastRootPath && filePath.startsWith(this.lastRootPath)) {
        relativePath = filePath.slice(this.lastRootPath.length);
        if (relativePath.startsWith('/')) relativePath = relativePath.slice(1);
      } else {
        // Fallback to basename
        relativePath = filePath.includes('/') ? (filePath.split('/').pop() || filePath) : filePath;
      }
      console.log(`[ApiClient] Calling API with path: ${relativePath} (from: ${filePath})`);
      
      const response = await fetch(`${this.baseUrl}/api/file/${encodeURIComponent(relativePath)}`);
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const raw = await response.json();
      console.log(`[ApiClient] Received response:`, raw);
      const data = {
        path: raw.path,
        content: raw.content,
        memos: this.transformApiMemos(raw.memos || [])
      };
      return { data };
    } catch (error) {
      console.error(`[ApiClient] Error fetching file:`, error);
      return {
        data: { path: filePath, content: '', memos: [] },
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  // Transform backend snake_case and Level representation to frontend camelCase + numeric levels starting at 1
  private transformApiMemos(memos: any[]): FunctionMemo[] {
    const normLevel = (lvl: any): number => {
      if (typeof lvl === 'number') return Math.max(1, lvl + 1);
      if (Array.isArray(lvl) && lvl.length > 0 && typeof lvl[0] === 'number') return Math.max(1, lvl[0] + 1);
      if (lvl && typeof lvl === 'object') {
        // tuple struct may serialize as {"0": n}
        if (typeof (lvl as any)[0] === 'number') return Math.max(1, ((lvl as any)[0] as number) + 1);
        if (typeof (lvl as any).level === 'number') return Math.max(1, ((lvl as any).level as number) + 1);
      }
      return 1;
    };

    const toMemo = (m: any): FunctionMemo => {
      const children = Array.isArray(m.children) ? m.children.map(toMemo) : [];
      const codeBlocksSrc = m.codeBlocks || m.code_blocks || [];
      const codeBlocks = codeBlocksSrc.map((cb: any) => ({
        language: cb.language,
        code: cb.code,
      }));
      return {
        level: normLevel(m.level),
        title: m.title || '',
        description: m.description ?? undefined,
        content: m.content || '',
        codeBlocks,
        children,
      };
    };

    return memos.map(toMemo);
  }

  // Convert API response to frontend types (recursively)
  convertToDirectoryStructure(
    apiData: ApiDirectoryTree,
    currentPath?: string
  ): DirectoryStructure {
    // Use path from API data, or provided currentPath
    const path = currentPath || apiData.path;
    // Remember last root/current path for relative file path computation
    this.lastRootPath = path;
    const items: FileItem[] = [];

    // Add subdirectories (recursively)
    apiData.subdirectories.forEach(subdir => {
      const dirName = subdir.path.split('/').pop() || subdir.path;
      const subdirStructure = this.convertToDirectoryStructure(subdir, subdir.path);
      items.push({
        name: dirName,
        path: subdir.path,
        type: 'directory',
        children: subdirStructure.items
      });
    });

    // Add files
    apiData.files.forEach(file => {
      const extension = this.getFileExtension(file);
      items.push({
        name: file,
        path: `${path}/${file}`,
        type: 'file',
        extension
      });
    });

    return {
      path,
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
  private mockData: ApiDirectoryTree = {
    path: '/mock',
    files: ['README.md', 'Cargo.toml', 'input.md', 'test_hierarchy.md'],
    subdirectories: [
      {
        path: '/mock/src',
        files: ['main.rs', 'lib.rs', 'parser.rs', 'server.rs'],
        subdirectories: []
      },
      {
        path: '/mock/frontend',
        files: ['package.json', 'vite.config.ts', 'tsconfig.json'],
        subdirectories: [
          {
            path: '/mock/frontend/src',
            files: ['App.tsx', 'main.tsx', 'index.css'],
            subdirectories: []
          }
        ]
      }
    ]
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

    return { data: this.mockData };
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
