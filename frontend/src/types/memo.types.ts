export interface MemoNode {
  id: string;
  title: string;
  level: number;
  description?: string;
  path?: string;
  content: string;
  codeBlocks: CodeBlock[];
  children: MemoNode[];
}

export interface CodeBlock {
  language: string;
  code: string;
}

export interface ViewMode {
  current: 'memo' | 'flow';
}

export interface ZoomState {
  zoom: number;
  panX: number;
  panY: number;
}

export interface ExpandedState {
  [nodeId: string]: boolean;
}

export interface AppState {
  memoNodes: MemoNode[];
  viewMode: ViewMode['current'];
  zoomState: ZoomState;
  expandedState: ExpandedState;
  selectedFile: string;
}