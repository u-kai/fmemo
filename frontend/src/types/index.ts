export interface CodeBlock {
  language: string;
  code: string;
}

export interface FunctionMemo {
  level: number;
  title: string;
  content: string;
  codeBlocks: CodeBlock[];
  children: FunctionMemo[];
}

export interface ZoomState {
  zoom: number;
  panX: number;
  panY: number;
}

export interface ViewMode {
  mode: 'memo' | 'flow';
  layout: 'vertical' | 'horizontal';
}

export interface FlowNode {
  title: string;
  level: number;
  description?: string;
  path?: string;
  children: FlowNode[];
}