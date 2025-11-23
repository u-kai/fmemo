import type { HTMLAttributes } from 'react';

interface IconProps extends HTMLAttributes<HTMLSpanElement> {
  name: 'expand' | 'zoom-in' | 'zoom-out' | 'reset' | 'fit' | 'menu';
  size?: 'sm' | 'md' | 'lg';
}

const iconPaths = {
  expand: '▶',
  'zoom-in': '+',
  'zoom-out': '−',
  reset: '↺',
  fit: '⌐',
  menu: '☰',
};

export const Icon = ({ 
  name, 
  size = 'md', 
  className = '', 
  ...props 
}: IconProps) => {
  const baseClasses = 'inline-block font-mono select-none transition-transform duration-200';
  
  const sizeClasses = {
    sm: 'text-xs',
    md: 'text-sm',
    lg: 'text-base',
  };

  const classes = `${baseClasses} ${sizeClasses[size]} ${className}`.trim();

  return (
    <span
      className={classes}
      {...props}
    >
      {iconPaths[name]}
    </span>
  );
};