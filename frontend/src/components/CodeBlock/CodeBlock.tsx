import React from 'react';
import type { CodeBlock as CodeBlockType } from '../../types';
import './CodeBlock.css';

interface CodeBlockProps {
  codeBlock: CodeBlockType;
}

export const CodeBlock: React.FC<CodeBlockProps> = ({ codeBlock }) => {
  return (
    <div className="code-block">
      {codeBlock.language && (
        <div className="code-language">{codeBlock.language}</div>
      )}
      <pre>
        <code>{codeBlock.code}</code>
      </pre>
    </div>
  );
};