import type { Meta, StoryObj } from '@storybook/react-vite';
import { fn } from 'storybook/test';
import { Button } from '../components/common/Button/Button';

const meta = {
  title: 'Components/Button',
  component: Button,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: { type: 'select' },
      options: ['zoom', 'mode'],
    },
    active: {
      control: { type: 'boolean' },
    },
  },
  args: { onClick: fn() },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ZoomButton: Story = {
  args: {
    variant: 'zoom',
    children: '+',
  },
};

export const ZoomButtonMinus: Story = {
  args: {
    variant: 'zoom',
    children: '−',
  },
};

export const ZoomButtonReset: Story = {
  args: {
    variant: 'zoom',
    children: 'Reset',
  },
};

export const ModeButton: Story = {
  args: {
    variant: 'mode',
    children: 'Memo',
  },
};

export const ModeButtonActive: Story = {
  args: {
    variant: 'mode',
    children: 'Flow',
    active: true,
  },
};