import type { Meta, StoryObj } from '@storybook/react-vite';
import { fn } from 'storybook/test';
import { ZoomControls } from '../components/ZoomControls/ZoomControls';

const meta = {
  title: 'Components/ZoomControls',
  component: ZoomControls,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
} satisfies Meta<typeof ZoomControls>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    zoomState: {
      zoom: 1.0,
      panX: 0,
      panY: 0,
    },
    viewMode: {
      mode: 'memo',
      layout: 'vertical',
    },
    onZoomIn: fn(),
    onZoomOut: fn(),
    onResetZoom: fn(),
    onFitToScreen: fn(),
    onModeChange: fn(),
  },
};

export const ZoomedIn: Story = {
  args: {
    zoomState: {
      zoom: 1.5,
      panX: 10,
      panY: 20,
    },
    viewMode: {
      mode: 'memo',
      layout: 'vertical',
    },
    onZoomIn: fn(),
    onZoomOut: fn(),
    onResetZoom: fn(),
    onFitToScreen: fn(),
    onModeChange: fn(),
  },
};

export const FlowMode: Story = {
  args: {
    zoomState: {
      zoom: 0.8,
      panX: 0,
      panY: 0,
    },
    viewMode: {
      mode: 'flow',
      layout: 'horizontal',
    },
    onZoomIn: fn(),
    onZoomOut: fn(),
    onResetZoom: fn(),
    onFitToScreen: fn(),
    onModeChange: fn(),
  },
};