import React from 'react';
import type { FlowNode as FlowNodeType } from '../../types';

interface FlowNodeProps {
  node: FlowNodeType;
  onNodeClick?: (node: FlowNodeType) => void;
  depth: number;
}

export const FlowNode: React.FC<FlowNodeProps> = ({ 
  node, 
  onNodeClick,
  depth 
}) => {
  const handleClick = () => {
    onNodeClick?.(node);
  };

  const levelClass = `level-${Math.min(node.level, 8)}`;
  const depthClass = `depth-${depth}`;

  return (
    <div 
      className={`flow-node ${levelClass} ${depthClass}`}
      onClick={handleClick}
    >
      <div className="flow-node-title">{node.title}</div>
      
      {node.description && (
        <div className="flow-node-description">{node.description}</div>
      )}
      
      {node.path && (
        <div className="flow-node-path">{node.path}</div>
      )}
    </div>
  );
};