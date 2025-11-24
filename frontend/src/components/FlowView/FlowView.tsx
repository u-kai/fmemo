import React, { useEffect, useRef } from 'react';
import type { FunctionMemo, FlowNode as FlowNodeType } from '../../types';
import { FlowNode } from './FlowNode';
import './FlowView.css';

interface FlowViewProps {
  memos: FunctionMemo[];
  onNodeClick?: (node: FlowNodeType) => void;
  className?: string;
}

export const FlowView: React.FC<FlowViewProps> = ({ 
  memos, 
  onNodeClick,
  className = '' 
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const flowNodes = convertMemosToFlowNodes(memos);

  useEffect(() => {
    // Add connecting lines after component mounts
    const timer = setTimeout(() => {
      if (containerRef.current) {
        addConnectingLines(containerRef.current);
      }
    }, 100);

    return () => clearTimeout(timer);
  }, [memos]);

  return (
    <div 
      id="flow-view" 
      className={`flow-view ${className}`}
      ref={containerRef}
    >
      <div className="flow-diagram-container">
        {renderHierarchicalFlow(flowNodes, 0, onNodeClick)}
      </div>
    </div>
  );
};

function convertMemosToFlowNodes(memos: FunctionMemo[]): FlowNodeType[] {
  return memos.map(memo => convertMemoToFlowNode(memo));
}

function convertMemoToFlowNode(memo: FunctionMemo): FlowNodeType {
  // Extract description and path from content
  let description = '';
  let path = '';
  
  const content = memo.content;
  
  // Try to extract description from first line or paragraph
  const lines = content.split('\n').filter(line => line.trim());
  if (lines.length > 0) {
    description = lines[0].substring(0, 100);
    if (description.length < lines[0].length) {
      description += '...';
    }
  }
  
  // Try to extract file path if mentioned
  const pathMatch = content.match(/(?:file|path|from):\s*([^\s\n]+)/i);
  if (pathMatch) {
    path = pathMatch[1];
  }

  return {
    title: memo.title,
    level: memo.level,
    description,
    path,
    children: memo.children.map(child => convertMemoToFlowNode(child))
  };
}

function renderHierarchicalFlow(
  nodes: FlowNodeType[], 
  depth: number, 
  onNodeClick?: (node: FlowNodeType) => void
): React.ReactNode {
  if (!nodes || nodes.length === 0) return null;

  if (nodes.length === 1) {
    const node = nodes[0];
    const isTopLevel = node.level === 1 ? ' top-level' : '';
    
    return (
      <div className={`flow-tree-node${isTopLevel}`} key={`${node.title}-${depth}`}>
        <FlowNode 
          node={node} 
          onNodeClick={onNodeClick}
          depth={depth}
        />
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
        const isTopLevel = node.level === 1 ? ' top-level' : '';
        
        return (
          <div 
            className={`flow-tree-node sibling${isTopLevel}`} 
            key={`${node.title}-${index}-${depth}`}
          >
            <FlowNode 
              node={node} 
              onNodeClick={onNodeClick}
              depth={depth}
            />
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

function addConnectingLines(container: HTMLElement) {
  // Remove existing lines
  container.querySelectorAll('.flow-connection-line').forEach(line => line.remove());
  
  // Add parent-child lines (vertical)
  container.querySelectorAll('.flow-tree-node').forEach(treeNode => {
    const parentNode = treeNode.querySelector(':scope > .flow-node') as HTMLElement;
    const childrenFlow = treeNode.querySelector(':scope > .children-flow') as HTMLElement;
    
    if (parentNode && childrenFlow) {
      const childNodes = childrenFlow.querySelectorAll(':scope > .flow-tree-node > .flow-node, :scope > .siblings-flow > .flow-tree-node > .flow-node') as NodeListOf<HTMLElement>;
      
      if (childNodes.length > 0) {
        childNodes.forEach(childNode => {
          createVerticalLine(container, parentNode, childNode);
        });
      }
    }
  });
  
  // Add sibling lines (horizontal) - skip level 1 siblings
  container.querySelectorAll('.siblings-flow').forEach(siblingsContainer => {
    const siblingNodes = siblingsContainer.querySelectorAll(':scope > .flow-tree-node > .flow-node') as NodeListOf<HTMLElement>;
    
    // Check if any of the siblings are level-1 (top level)
    let hasLevel1 = false;
    siblingNodes.forEach(node => {
      if (node.classList.contains('level-1')) {
        hasLevel1 = true;
      }
    });
    
    // Only add horizontal lines if not level-1 siblings
    if (!hasLevel1) {
      for (let i = 0; i < siblingNodes.length - 1; i++) {
        createHorizontalLine(container, siblingNodes[i], siblingNodes[i + 1]);
      }
    }
  });
}

function createVerticalLine(container: HTMLElement, parentNode: HTMLElement, childNode: HTMLElement) {
  const parentRect = parentNode.getBoundingClientRect();
  const childRect = childNode.getBoundingClientRect();
  const containerRect = container.getBoundingClientRect();
  
  const line = document.createElement('div');
  line.className = 'flow-connection-line vertical-line';
  
  const startX = parentRect.left + parentRect.width / 2 - containerRect.left;
  const startY = parentRect.bottom - containerRect.top;
  const endY = childRect.top - containerRect.top;
  
  line.style.position = 'absolute';
  line.style.left = `${startX - 1}px`;
  line.style.top = `${startY}px`;
  line.style.width = '2px';
  line.style.height = `${endY - startY}px`;
  line.style.background = '#34495e';
  line.style.zIndex = '1';
  
  container.appendChild(line);
  
  // Add arrow at the end
  const arrow = document.createElement('div');
  arrow.className = 'flow-arrow down-arrow';
  arrow.style.position = 'absolute';
  arrow.style.left = `${startX - 6}px`;
  arrow.style.top = `${endY - 10}px`;
  arrow.style.width = '0';
  arrow.style.height = '0';
  arrow.style.borderLeft = '6px solid transparent';
  arrow.style.borderRight = '6px solid transparent';
  arrow.style.borderTop = '10px solid #34495e';
  arrow.style.zIndex = '2';
  
  container.appendChild(arrow);
}

function createHorizontalLine(container: HTMLElement, leftNode: HTMLElement, rightNode: HTMLElement) {
  const leftRect = leftNode.getBoundingClientRect();
  const rightRect = rightNode.getBoundingClientRect();
  const containerRect = container.getBoundingClientRect();
  
  const line = document.createElement('div');
  line.className = 'flow-connection-line horizontal-line';
  
  const startX = leftRect.right - containerRect.left;
  const endX = rightRect.left - containerRect.left;
  const y = leftRect.top + leftRect.height / 2 - containerRect.top;
  
  line.style.position = 'absolute';
  line.style.left = `${startX}px`;
  line.style.top = `${y - 1}px`;
  line.style.width = `${endX - startX}px`;
  line.style.height = '2px';
  line.style.background = '#95a5a6';
  line.style.zIndex = '1';
  
  container.appendChild(line);
  
  // Add arrow at the end
  const arrow = document.createElement('div');
  arrow.className = 'flow-arrow right-arrow';
  arrow.style.position = 'absolute';
  arrow.style.left = `${endX - 10}px`;
  arrow.style.top = `${y - 6}px`;
  arrow.style.width = '0';
  arrow.style.height = '0';
  arrow.style.borderTop = '6px solid transparent';
  arrow.style.borderBottom = '6px solid transparent';
  arrow.style.borderLeft = '10px solid #95a5a6';
  arrow.style.zIndex = '2';
  
  container.appendChild(arrow);
}