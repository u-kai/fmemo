import type { Meta, StoryObj } from '@storybook/react-vite';
import { fn } from 'storybook/test';
import { FlowView } from '../components/FlowView/FlowView';
import type { FunctionMemo } from '../types';

const meta = {
  title: 'Components/FlowView',
  component: FlowView,
  parameters: {
    layout: 'fullscreen',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof FlowView>;

export default meta;
type Story = StoryObj<typeof meta>;

const sampleMemos: FunctionMemo[] = [
  {
    level: 1,
    title: 'Main Application',
    content: 'The main application entry point that coordinates all other components.\n\n- Initialize system\n- Setup configuration\n- Start main loop',
    codeBlocks: [
      {
        language: 'typescript',
        code: `function main() {
  const app = new Application();
  app.initialize();
  app.start();
}`
      }
    ],
    children: [
      {
        level: 2,
        title: 'System Initialization',
        content: 'path: /src/init.ts\n\nInitializes all system components and prepares the application for startup.',
        codeBlocks: [],
        children: [
          {
            level: 3,
            title: 'Database Setup',
            content: 'Configures database connections and runs migrations.',
            codeBlocks: [],
            children: []
          },
          {
            level: 3,
            title: 'Logger Configuration',
            content: 'Sets up logging infrastructure for the application.',
            codeBlocks: [],
            children: []
          }
        ]
      },
      {
        level: 2,
        title: 'Main Loop',
        content: 'path: /src/loop.ts\n\nThe core application loop that processes events and updates the system.',
        codeBlocks: [],
        children: [
          {
            level: 3,
            title: 'Event Processing',
            content: 'Handles incoming events from various sources.',
            codeBlocks: [],
            children: []
          },
          {
            level: 3,
            title: 'State Updates',
            content: 'Updates application state based on processed events.',
            codeBlocks: [],
            children: []
          },
          {
            level: 3,
            title: 'Rendering',
            content: 'Renders the current state to the user interface.',
            codeBlocks: [],
            children: []
          }
        ]
      }
    ]
  },
  {
    level: 1,
    title: 'Utility Functions',
    content: 'Collection of utility functions used throughout the application.',
    codeBlocks: [],
    children: [
      {
        level: 2,
        title: 'String Utils',
        content: 'path: /src/utils/strings.ts\n\nString manipulation and formatting utilities.',
        codeBlocks: [],
        children: []
      },
      {
        level: 2,
        title: 'Date Utils',
        content: 'path: /src/utils/dates.ts\n\nDate formatting and manipulation functions.',
        codeBlocks: [],
        children: []
      }
    ]
  }
];

const simpleMemos: FunctionMemo[] = [
  {
    level: 1,
    title: 'Simple Function',
    content: 'A basic function with minimal complexity.',
    codeBlocks: [],
    children: [
      {
        level: 2,
        title: 'Helper Method',
        content: 'A helper method used by the main function.',
        codeBlocks: [],
        children: []
      }
    ]
  }
];

export const ComplexHierarchy: Story = {
  args: {
    memos: sampleMemos,
    onNodeClick: fn(),
  },
};

export const SimpleStructure: Story = {
  args: {
    memos: simpleMemos,
    onNodeClick: fn(),
  },
};

export const EmptyState: Story = {
  args: {
    memos: [],
    onNodeClick: fn(),
  },
};

export const SingleMemo: Story = {
  args: {
    memos: [sampleMemos[0]],
    onNodeClick: fn(),
  },
};