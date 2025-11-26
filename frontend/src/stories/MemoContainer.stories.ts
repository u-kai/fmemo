import type { Meta, StoryObj } from '@storybook/react-vite';
import { MemoContainer } from '../components/MemoContainer/MemoContainer';
import type { FunctionMemo } from '../types';

const meta = {
  title: 'Components/MemoContainer',
  component: MemoContainer,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof MemoContainer>;

export default meta;
type Story = StoryObj<typeof meta>;

const simpleMemo: FunctionMemo = {
  level: 1,
  title: 'Simple Function',
  content: 'This is a simple function that does something important.\n\n- First step\n- Second step\n- Third step',
  codeBlocks: [
    {
      language: 'typescript',
      code: `function simpleFunction(input: string): string {
  return input.toUpperCase();
}`
    }
  ],
  children: []
};

const nestedMemo: FunctionMemo = {
  level: 1,
  title: 'Parent Function',
  content: 'This is a parent function with nested children.',
  codeBlocks: [],
  children: [
    {
      level: 2,
      title: 'Child Function 1',
      content: 'This is the first child function.',
      codeBlocks: [
        {
          language: 'javascript',
          code: `function childOne() {
  console.log('Child One');
}`
        }
      ],
      children: []
    },
    {
      level: 2,
      title: 'Child Function 2',
      content: 'This is the second child function with its own child.',
      codeBlocks: [],
      children: [
        {
          level: 3,
          title: 'Grandchild Function',
          content: 'This is a deeply nested function.',
          codeBlocks: [
            {
              language: 'rust',
              code: `fn grandchild() {
    println!("Grandchild function");
}`
            }
          ],
          children: []
        }
      ]
    }
  ]
};

const level2Memo: FunctionMemo = {
  level: 2,
  title: 'Level 2 Function',
  content: 'This is a level 2 function with different styling.',
  codeBlocks: [],
  children: []
};

const level3Memo: FunctionMemo = {
  level: 3,
  title: 'Level 3 Function',
  content: 'This is a level 3 function.',
  codeBlocks: [],
  children: []
};

export const Simple: Story = {
  args: {
    memo: simpleMemo,
  },
};

export const WithChildren: Story = {
  args: {
    memo: nestedMemo,
  },
};

export const Level2: Story = {
  args: {
    memo: level2Memo,
  },
};

export const Level3: Story = {
  args: {
    memo: level3Memo,
  },
};

export const HorizontalLayout: Story = {
  args: {
    memo: nestedMemo,
    isHorizontal: true,
  },
};