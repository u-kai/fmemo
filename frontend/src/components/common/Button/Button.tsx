import React from 'react';

interface ButtonProps {
  onClick: () => void;
  children: React.ReactNode;
  variant?: 'zoom' | 'mode';
  active?: boolean;
  className?: string;
}

export const Button: React.FC<ButtonProps> = ({ 
  onClick, 
  children, 
  variant = 'zoom',
  active = false,
  className = ''
}) => {
  const baseClass = variant === 'mode' ? 'mode-btn' : 'zoom-btn';
  const activeClass = active && variant === 'mode' ? ' active' : '';
  
  return (
    <button 
      className={`${baseClass}${activeClass} ${className}`}
      onClick={onClick}
    >
      {children}
    </button>
  );
};