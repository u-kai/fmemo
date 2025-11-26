import { describe, it, expect, beforeEach, vi } from 'vitest';
import { MockApiClient } from './client';

describe('MockApiClient', () => {
  let client: MockApiClient;

  beforeEach(() => {
    client = new MockApiClient();
  });

  describe('getDirectoryTree', () => {
    it('returns root directory structure', async () => {
      const result = await client.getDirectoryTree('');
      
      expect(result.data.files).toContain('README.md');
      expect(result.data.files).toContain('Cargo.toml');
      expect(result.data.directories).toContain('src');
      expect(result.data.directories).toContain('frontend');
      expect(result.error).toBeUndefined();
    });

    it('returns src directory structure', async () => {
      const result = await client.getDirectoryTree('src');
      
      expect(result.data.files).toContain('main.rs');
      expect(result.data.files).toContain('lib.rs');
      expect(result.data.files).toContain('parser.rs');
      expect(result.data.directories).toHaveLength(0);
      expect(result.error).toBeUndefined();
    });

    it('returns empty structure for unknown path', async () => {
      const result = await client.getDirectoryTree('nonexistent');
      
      expect(result.data.files).toHaveLength(0);
      expect(result.data.directories).toHaveLength(0);
      expect(result.error).toBeUndefined();
    });
  });

  describe('getFileContent', () => {
    it('returns README.md content with memos', async () => {
      const result = await client.getFileContent('README.md');
      
      expect(result.data.path).toBe('README.md');
      expect(result.data.content).toContain('Function Memo Viewer');
      expect(result.data.memos).toHaveLength(1);
      expect(result.data.memos[0].title).toBe('Function Memo Viewer');
      expect(result.data.memos[0].children).toHaveLength(1);
      expect(result.error).toBeUndefined();
    });

    it('returns main.rs content with memos', async () => {
      const result = await client.getFileContent('src/main.rs');
      
      expect(result.data.path).toBe('src/main.rs');
      expect(result.data.content).toContain('Starting Function Memo server');
      expect(result.data.memos).toHaveLength(1);
      expect(result.data.memos[0].title).toBe('Main Function');
      expect(result.data.memos[0].codeBlocks).toHaveLength(1);
      expect(result.data.memos[0].codeBlocks[0].language).toBe('rust');
      expect(result.error).toBeUndefined();
    });

    it('returns input.md content with hierarchical structure', async () => {
      const result = await client.getFileContent('input.md');
      
      expect(result.data.memos).toHaveLength(1);
      
      const rootMemo = result.data.memos[0];
      expect(rootMemo.title).toBe('Sample Input File');
      expect(rootMemo.children).toHaveLength(1);
      
      const mainFunction = rootMemo.children[0];
      expect(mainFunction.title).toBe('Main Function');
      expect(mainFunction.level).toBe(2);
      expect(mainFunction.codeBlocks).toHaveLength(1);
      expect(mainFunction.children).toHaveLength(1);
      
      const helperFunctions = mainFunction.children[0];
      expect(helperFunctions.title).toBe('Helper Functions');
      expect(helperFunctions.level).toBe(3);
      expect(helperFunctions.children).toHaveLength(1);
      
      const processData = helperFunctions.children[0];
      expect(processData.title).toBe('Process Data');
      expect(processData.level).toBe(4);
      expect(result.error).toBeUndefined();
    });

    it('returns default content for unknown files', async () => {
      const result = await client.getFileContent('unknown.txt');
      
      expect(result.data.path).toBe('unknown.txt');
      expect(result.data.content).toContain('unknown.txt');
      expect(result.data.memos).toHaveLength(1);
      expect(result.data.memos[0].title).toBe('Content from unknown.txt');
      expect(result.error).toBeUndefined();
    });

    it('simulates network delay', async () => {
      const start = Date.now();
      await client.getFileContent('README.md');
      const elapsed = Date.now() - start;
      
      expect(elapsed).toBeGreaterThanOrEqual(300);
    });
  });

  describe('convertToDirectoryStructure', () => {
    it('converts API response to frontend format', () => {
      const apiData = {
        files: ['test.md', 'config.json'],
        directories: ['src', 'docs']
      };
      
      const result = client.convertToDirectoryStructure(apiData, '/test');
      
      expect(result.path).toBe('/test');
      expect(result.items).toHaveLength(4);
      
      // Directories should come first
      expect(result.items[0].type).toBe('directory');
      expect(result.items[0].name).toBe('docs');
      expect(result.items[1].type).toBe('directory');
      expect(result.items[1].name).toBe('src');
      
      // Then files
      expect(result.items[2].type).toBe('file');
      expect(result.items[2].extension).toBe('.json');
      expect(result.items[3].type).toBe('file');
      expect(result.items[3].extension).toBe('.md');
    });

    it('handles empty API response', () => {
      const apiData = {
        files: [],
        directories: []
      };
      
      const result = client.convertToDirectoryStructure(apiData);
      
      expect(result.path).toBe('/Users/kai/refactor-fmemo');
      expect(result.items).toHaveLength(0);
    });
  });
});