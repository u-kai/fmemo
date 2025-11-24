import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { FunctionMemo, FlowNode as FlowNodeType } from "../../types";
import { FlowNode } from "./FlowNode";
import "./FlowView.css";

interface FlowViewProps {
  memos: FunctionMemo[];
  onNodeClick?: (node: FlowNodeType) => void;
  className?: string;
  // Current zoom level applied to container (1.0 means no scaling)
  zoom?: number;
}

export const FlowView: React.FC<FlowViewProps> = ({
  memos,
  onNodeClick,
  className = "",
  zoom = 1.0,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const flowNodes = convertMemosToFlowNodes(memos);
  const [connectors, setConnectors] = useState<Connector[]>([]);

  // Helper: compute connectors after layout settles (next frames)
  const scheduleCompute = (el: HTMLElement) => {
    // Run after 2 RAFs to ensure styles/fonts/layout applied
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        setConnectors(computeConnectors(el));
      });
    });
  };

  // Compute connectors and SVG size before paint
  useLayoutEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    scheduleCompute(el);
  }, [memos]);

  // Recompute on resize
  useEffect(() => {
    const handleResize = () => {
      const el = containerRef.current;
      if (!el) return;
      scheduleCompute(el);
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // Recompute on zoom changes to sync with any transform transition
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    scheduleCompute(el);
  }, [zoom]);

  // Observe size/content changes within the flow container
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const resizeObserver = new ResizeObserver(() => scheduleCompute(el));
    const mutationObserver = new MutationObserver(() => scheduleCompute(el));
    resizeObserver.observe(el);
    mutationObserver.observe(el, { childList: true, subtree: true });
    return () => {
      resizeObserver.disconnect();
      mutationObserver.disconnect();
    };
  }, []);

  return (
    <div id="flow-view" className={`flow-view ${className}`} ref={containerRef}>
      <svg
        className="flow-svg"
        width="100%"
        height="100%"
        style={{ position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'visible' }}
      >
        <defs>
          <marker id="arrow-dark" viewBox="0 0 10 10" refX="10" refY="5" markerUnits="strokeWidth" markerWidth="6" markerHeight="6" orient="auto">
            <path d="M0,0 L10,5 L0,10 z" fill="#34495e" />
          </marker>
          <marker id="arrow-light" viewBox="0 0 10 10" refX="10" refY="5" markerUnits="strokeWidth" markerWidth="6" markerHeight="6" orient="auto">
            <path d="M0,0 L10,5 L0,10 z" fill="#95a5a6" />
          </marker>
        </defs>
        {connectors.map((c, i) => (
          <line
            key={i}
            x1={c.x1}
            y1={c.y1}
            x2={c.x2}
            y2={c.y2}
            stroke={c.type === 'vertical' ? '#34495e' : '#95a5a6'}
            strokeWidth={2}
            markerEnd={c.arrow === 'dark' ? 'url(#arrow-dark)' : c.arrow === 'light' ? 'url(#arrow-light)' : undefined}
          />
        ))}
      </svg>
      <div className="flow-diagram-container">
        {renderHierarchicalFlow(flowNodes, 0, onNodeClick)}
      </div>
    </div>
  );
};

function convertMemosToFlowNodes(memos: FunctionMemo[]): FlowNodeType[] {
  return memos.map((memo) => convertMemoToFlowNode(memo));
}

function convertMemoToFlowNode(memo: FunctionMemo): FlowNodeType {
  // Extract description and path from content

  return {
    title: memo.title,
    level: memo.level,
    description: memo.description,
    path: "",
    children: memo.children.map((child) => convertMemoToFlowNode(child)),
  };
}

function renderHierarchicalFlow(
  nodes: FlowNodeType[],
  depth: number,
  onNodeClick?: (node: FlowNodeType) => void,
): React.ReactNode {
  if (!nodes || nodes.length === 0) return null;

  if (nodes.length === 1) {
    const node = nodes[0];
    const isTopLevel = node.level === 1 ? " top-level" : "";

    return (
      <div
        className={`flow-tree-node${isTopLevel}`}
        key={`${node.title}-${depth}`}
      >
        <FlowNode node={node} onNodeClick={onNodeClick} depth={depth} />
        {node.children.length > 0 && (
          <div className="children-flow">
            {renderHierarchicalFlow(node.children, depth + 1, onNodeClick)}
          </div>
        )}
      </div>
    );
  }

  // Multiple siblings
  return (
    <div className="siblings-flow" key={`siblings-${depth}`}>
      {nodes.map((node, index) => {
        const isTopLevel = node.level === 1 ? " top-level" : "";

        return (
          <div
            className={`flow-tree-node sibling${isTopLevel}`}
            key={`${node.title}-${index}-${depth}`}
          >
            <FlowNode node={node} onNodeClick={onNodeClick} depth={depth} />
            {node.children.length > 0 && (
              <div className="children-flow">
                {renderHierarchicalFlow(node.children, depth + 1, onNodeClick)}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

type Connector = { type: 'vertical' | 'horizontal'; x1: number; y1: number; x2: number; y2: number; arrow?: 'dark' | 'light' };

function computeConnectors(container: HTMLElement): Connector[] {
  const connectors: Connector[] = [];
  const containerRect = container.getBoundingClientRect();

  // Parent-child vertical connectors
  container.querySelectorAll(".flow-tree-node").forEach((treeNode) => {
    const parentNode = treeNode.querySelector(":scope > .flow-node") as HTMLElement | null;
    const childrenFlow = treeNode.querySelector(":scope > .children-flow") as HTMLElement | null;
    if (!parentNode || !childrenFlow) return;

    const childNodes = childrenFlow.querySelectorAll(
      ":scope > .flow-tree-node > .flow-node, :scope > .siblings-flow > .flow-tree-node > .flow-node",
    ) as NodeListOf<HTMLElement>;
    if (childNodes.length === 0) return;

    childNodes.forEach((childNode, index) => {
      const parentRect = parentNode.getBoundingClientRect();
      const childRect = childNode.getBoundingClientRect();
      const x = parentRect.left + parentRect.width / 2 - containerRect.left;
      const y1 = parentRect.bottom - containerRect.top;
      const y2 = childRect.top - containerRect.top;
      connectors.push({ type: 'vertical', x1: x, y1, x2: x, y2, arrow: index === 0 ? 'dark' : undefined });
    });
  });

  // Sibling horizontal connectors (skip level-1 siblings)
  container.querySelectorAll(".siblings-flow").forEach((siblingsContainer) => {
    const siblingNodes = siblingsContainer.querySelectorAll(
      ":scope > .flow-tree-node > .flow-node",
    ) as NodeListOf<HTMLElement>;
    if (siblingNodes.length < 2) return;

    let hasLevel1 = false;
    siblingNodes.forEach((node) => {
      if (node.classList.contains("level-1")) hasLevel1 = true;
    });
    if (hasLevel1) return;

    for (let i = 0; i < siblingNodes.length - 1; i++) {
      const leftRect = siblingNodes[i].getBoundingClientRect();
      const rightRect = siblingNodes[i + 1].getBoundingClientRect();
      const startX = leftRect.right - containerRect.left;
      const endX = rightRect.left - containerRect.left;
      const y = leftRect.top + leftRect.height / 2 - containerRect.top;
      connectors.push({ type: 'horizontal', x1: startX, y1: y, x2: endX, y2: y, arrow: 'light' });
    }
  });

  return connectors;
}

// Switched to SVG overlay; DOM-based line drawing removed.
