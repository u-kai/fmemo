import type { ReactNode } from 'react';
import { Badge } from '../atoms/Badge';
import { Icon } from '../atoms/Icon';

interface NodeTitleProps {
  title: string;
  level: number;
  description?: string;
  path?: string;
  isExpanded?: boolean;
  hasChildren?: boolean;
  onToggle?: () => void;
  children?: ReactNode;
}

export const NodeTitle = ({
  title,
  level,
  description,
  path,
  isExpanded = false,
  hasChildren = false,
  onToggle,
  children,
}: NodeTitleProps) => {
  const levelVariant = `level-${Math.min(level, 5)}` as 'level-1' | 'level-2' | 'level-3' | 'level-4' | 'level-5';

  return (
    <div className="w-full">
      <div 
        className="flex items-center gap-2 cursor-pointer hover:bg-gray-50 p-2 rounded transition-colors"
        onClick={onToggle}
      >
        {hasChildren && (
          <Icon 
            name="expand" 
            size="sm"
            className={`transform transition-transform ${isExpanded ? 'rotate-90' : ''}`}
          />
        )}
        
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <Badge variant={levelVariant} size="sm">
              H{level}
            </Badge>
            <h1 className="font-bold text-gray-900 truncate m-0">
              {title}
            </h1>
          </div>
          
          {description && (
            <div className="mb-1">
              <Badge variant="description" size="sm">
                {description}
              </Badge>
            </div>
          )}
          
          {path && (
            <Badge variant="path" size="sm">
              {path}
            </Badge>
          )}
        </div>
        
        {children}
      </div>
    </div>
  );
};