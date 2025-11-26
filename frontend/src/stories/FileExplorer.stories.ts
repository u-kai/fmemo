import type { Meta, StoryObj } from '@storybook/react-vite';
import { fn } from 'storybook/test';
import { FileExplorer } from '../components/FileExplorer/FileExplorer';
import type { DirectoryStructure } from '../types';

const meta = {
  title: 'Components/FileExplorer',
  component: FileExplorer,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof FileExplorer>;

export default meta;
type Story = StoryObj<typeof meta>;

const sampleDirectoryStructure: DirectoryStructure = {
  path: '/Users/kai/refactor-fmemo',
  items: [
    {
      name: 'src',
      path: '/Users/kai/refactor-fmemo/src',
      type: 'directory',
      children: [
        {
          name: 'main.rs',
          path: '/Users/kai/refactor-fmemo/src/main.rs',
          type: 'file',
          extension: '.rs'
        },
        {
          name: 'lib.rs',
          path: '/Users/kai/refactor-fmemo/src/lib.rs',
          type: 'file',
          extension: '.rs'
        },
        {
          name: 'parser.rs',
          path: '/Users/kai/refactor-fmemo/src/parser.rs',
          type: 'file',
          extension: '.rs'
        }
      ]
    },
    {
      name: 'frontend',
      path: '/Users/kai/refactor-fmemo/frontend',
      type: 'directory',
      children: [
        {
          name: 'src',
          path: '/Users/kai/refactor-fmemo/frontend/src',
          type: 'directory',
          children: [
            {
              name: 'App.tsx',
              path: '/Users/kai/refactor-fmemo/frontend/src/App.tsx',
              type: 'file',
              extension: '.tsx'
            },
            {
              name: 'components',
              path: '/Users/kai/refactor-fmemo/frontend/src/components',
              type: 'directory',
              children: [
                {
                  name: 'MemoViewer.tsx',
                  path: '/Users/kai/refactor-fmemo/frontend/src/components/MemoViewer.tsx',
                  type: 'file',
                  extension: '.tsx'
                },
                {
                  name: 'FileExplorer.tsx',
                  path: '/Users/kai/refactor-fmemo/frontend/src/components/FileExplorer.tsx',
                  type: 'file',
                  extension: '.tsx'
                }
              ]
            }
          ]
        },
        {
          name: 'package.json',
          path: '/Users/kai/refactor-fmemo/frontend/package.json',
          type: 'file',
          extension: '.json'
        }
      ]
    },
    {
      name: 'README.md',
      path: '/Users/kai/refactor-fmemo/README.md',
      type: 'file',
      extension: '.md'
    },
    {
      name: 'test_hierarchy.md',
      path: '/Users/kai/refactor-fmemo/test_hierarchy.md',
      type: 'file',
      extension: '.md'
    },
    {
      name: 'input.md',
      path: '/Users/kai/refactor-fmemo/input.md',
      type: 'file',
      extension: '.md'
    }
  ]
};

const smallDirectoryStructure: DirectoryStructure = {
  path: '/simple-project',
  items: [
    {
      name: 'index.html',
      path: '/simple-project/index.html',
      type: 'file',
      extension: '.html'
    },
    {
      name: 'style.css',
      path: '/simple-project/style.css',
      type: 'file',
      extension: '.css'
    },
    {
      name: 'script.js',
      path: '/simple-project/script.js',
      type: 'file',
      extension: '.js'
    }
  ]
};

export const Default: Story = {
  args: {
    directoryStructure: sampleDirectoryStructure,
    onFileSelect: fn(),
  },
};

export const WithSelectedFile: Story = {
  args: {
    directoryStructure: sampleDirectoryStructure,
    onFileSelect: fn(),
    selectedFile: '/Users/kai/refactor-fmemo/src/main.rs',
  },
};

export const SimpleProject: Story = {
  args: {
    directoryStructure: smallDirectoryStructure,
    onFileSelect: fn(),
  },
};

export const WithSelectedMarkdown: Story = {
  args: {
    directoryStructure: sampleDirectoryStructure,
    onFileSelect: fn(),
    selectedFile: '/Users/kai/refactor-fmemo/README.md',
  },
};