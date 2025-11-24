export interface CodeBlock {
  language: string;
  code: string;
}

export interface FunctionMemo {
  level: number;
  title: string;
  description?: string;
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
  mode: "memo" | "flow";
  layout: "vertical" | "horizontal";
}

export interface FlowNode {
  title: string;
  level: number;
  description?: string;
  path?: string;
  children: FlowNode[];
}

export interface FileItem {
  name: string;
  path: string;
  type: "file" | "directory";
  extension?: string;
  children?: FileItem[];
}

export interface DirectoryStructure {
  path: string;
  items: FileItem[];
}

