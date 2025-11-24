import { Button } from '../common/Button/Button';
import type { ViewMode, ZoomState } from '../../types';
import './ZoomControls.css';

interface ZoomControlsProps {
  zoomState: ZoomState;
  viewMode: ViewMode;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onResetZoom: () => void;
  onFitToScreen: () => void;
  onModeChange: (mode: 'memo' | 'flow') => void;
}

export const ZoomControls: React.FC<ZoomControlsProps> = ({
  zoomState,
  viewMode,
  onZoomIn,
  onZoomOut,
  onResetZoom,
  onFitToScreen,
  onModeChange
}) => {
  return (
    <div id="zoom-controls">
      <Button variant="zoom" onClick={onZoomOut}>−</Button>
      <span id="zoom-level">{Math.round(zoomState.zoom * 100)}%</span>
      <Button variant="zoom" onClick={onZoomIn}>+</Button>
      <Button variant="zoom" onClick={onResetZoom}>Reset</Button>
      <Button variant="zoom" onClick={onFitToScreen}>Fit</Button>
      <div className="control-separator" />
      <Button 
        variant="mode" 
        active={viewMode.mode === 'memo'}
        onClick={() => onModeChange('memo')}
      >
        Memo
      </Button>
      <Button 
        variant="mode"
        active={viewMode.mode === 'flow'}
        onClick={() => onModeChange('flow')}
      >
        Flow
      </Button>
    </div>
  );
};