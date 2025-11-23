import { Button } from '../atoms/Button';

interface ModeToggleProps {
  currentMode: 'memo' | 'flow';
  onModeChange: (mode: 'memo' | 'flow') => void;
}

export const ModeToggle = ({ currentMode, onModeChange }: ModeToggleProps) => {
  return (
    <div className="flex gap-0 bg-white rounded-lg shadow-md border overflow-hidden">
      <Button
        variant={currentMode === 'memo' ? 'primary' : 'outline'}
        size="sm"
        onClick={() => onModeChange('memo')}
        className="rounded-none border-0"
      >
        Memo
      </Button>
      <Button
        variant={currentMode === 'flow' ? 'primary' : 'outline'}
        size="sm" 
        onClick={() => onModeChange('flow')}
        className="rounded-none border-0 border-l"
      >
        Flow
      </Button>
    </div>
  );
};