import type { Meta, StoryObj } from '@storybook/react-vite';
import { CodeBlock } from '../components/CodeBlock/CodeBlock';

const meta = {
  title: 'Components/CodeBlock',
  component: CodeBlock,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof CodeBlock>;

export default meta;
type Story = StoryObj<typeof meta>;

export const TypeScriptCode: Story = {
  args: {
    codeBlock: {
      language: 'typescript',
      code: `interface User {
  id: number;
  name: string;
  email: string;
}

const user: User = {
  id: 1,
  name: 'John Doe',
  email: 'john@example.com'
};`,
    },
  },
};

export const JavaScriptCode: Story = {
  args: {
    codeBlock: {
      language: 'javascript',
      code: `function greetUser(name) {
  return \`Hello, \${name}!\`;
}

console.log(greetUser('World'));`,
    },
  },
};

export const RustCode: Story = {
  args: {
    codeBlock: {
      language: 'rust',
      code: `fn main() {
    let message = "Hello, World!";
    println!("{}", message);
    
    let numbers = vec![1, 2, 3, 4, 5];
    for number in numbers {
        println!("{}", number);
    }
}`,
    },
  },
};

export const NoLanguage: Story = {
  args: {
    codeBlock: {
      language: '',
      code: `This is a plain code block
without any language specified.

It should still be properly formatted
and displayed in a monospace font.`,
    },
  },
};