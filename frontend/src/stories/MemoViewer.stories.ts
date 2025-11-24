import type { Meta, StoryObj } from '@storybook/react-vite';
import { MemoViewer } from '../components/MemoViewer/MemoViewer';
import type { FunctionMemo } from '../types';

const meta = {
  title: 'Components/MemoViewer',
  component: MemoViewer,
  parameters: {
    layout: 'fullscreen',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof MemoViewer>;

export default meta;
type Story = StoryObj<typeof meta>;

const sampleMemos: FunctionMemo[] = [
  {
    level: 1,
    title: 'Main Function',
    content: 'This is the main function that orchestrates the application.\n\n- Initialize components\n- Setup event listeners\n- Start main loop',
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
        title: 'Initialize',
        content: 'Initialize all application components.',
        codeBlocks: [
          {
            language: 'typescript',
            code: `initialize(): void {
  this.setupEventListeners();
  this.loadConfiguration();
}`
          }
        ],
        children: []
      },
      {
        level: 2,
        title: 'Start',
        content: 'Start the main application loop.',
        codeBlocks: [],
        children: [
          {
            level: 3,
            title: 'Main Loop',
            content: 'The core application loop.',
            codeBlocks: [
              {
                language: 'typescript',
                code: `mainLoop(): void {
  while (this.running) {
    this.update();
    this.render();
  }
}`
              }
            ],
            children: []
          }
        ]
      }
    ]
  },
  {
    level: 1,
    title: 'Helper Functions',
    content: 'Various utility functions used throughout the application.',
    codeBlocks: [],
    children: [
      {
        level: 2,
        title: 'String Utils',
        content: 'String manipulation utilities.',
        codeBlocks: [
          {
            language: 'typescript',
            code: `function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}`
          }
        ],
        children: []
      }
    ]
  }
];

export const VerticalLayout: Story = {
  args: {
    memos: sampleMemos,
    viewMode: {
      mode: 'memo',
      layout: 'vertical',
    },
  },
};

export const HorizontalLayout: Story = {
  args: {
    memos: sampleMemos,
    viewMode: {
      mode: 'memo',
      layout: 'horizontal',
    },
  },
};

export const SingleMemo: Story = {
  args: {
    memos: [sampleMemos[0]],
    viewMode: {
      mode: 'memo',
      layout: 'vertical',
    },
  },
};