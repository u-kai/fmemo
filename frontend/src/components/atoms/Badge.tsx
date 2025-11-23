import type { HTMLAttributes, ReactNode } from 'react';

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  children: ReactNode;
  variant?: 'level-1' | 'level-2' | 'level-3' | 'level-4' | 'level-5' | 'path' | 'description';
  size?: 'sm' | 'md';
}

export const Badge = ({ 
  children, 
  variant = 'description', 
  size = 'sm',
  className = '',
  ...props 
}: BadgeProps) => {
  const baseClasses = 'inline-block px-2 py-1 rounded font-mono font-medium';
  
  const sizeClasses = {
    sm: 'text-xs',
    md: 'text-sm',
  };

  const variantClasses = {
    'level-1': 'bg-red-100 text-red-800 border border-red-300',
    'level-2': 'bg-blue-100 text-blue-800 border border-blue-300',
    'level-3': 'bg-green-100 text-green-800 border border-green-300',
    'level-4': 'bg-yellow-100 text-yellow-800 border border-yellow-300',
    'level-5': 'bg-purple-100 text-purple-800 border border-purple-300',
    'path': 'bg-gray-100 text-gray-600 border border-gray-300',
    'description': 'bg-gray-50 text-gray-700 italic',
  };

  const classes = `${baseClasses} ${sizeClasses[size]} ${variantClasses[variant]} ${className}`.trim();

  return (
    <span
      className={classes}
      {...props}
    >
      {children}
    </span>
  );
};