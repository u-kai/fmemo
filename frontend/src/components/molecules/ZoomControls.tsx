import { Button } from '../atoms/Button';
import { Icon } from '../atoms/Icon';

interface ZoomControlsProps {
  zoom: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onReset: () => void;
  onFit: () => void;
}

export const ZoomControls = ({
  zoom,
  onZoomIn,
  onZoomOut,
  onReset,
  onFit,
}: ZoomControlsProps) => {
  return (
    <div className="flex items-center gap-1 bg-white rounded-lg shadow-md p-1 border">
      <Button
        variant="outline"
        size="sm"
        onClick={onZoomOut}
        aria-label="Zoom out"
      >
        <Icon name="zoom-out" size="sm" />
      </Button>
      
      <span className="px-3 py-1 text-xs font-mono text-gray-700 min-w-16 text-center">
        {Math.round(zoom * 100)}%
      </span>
      
      <Button
        variant="outline" 
        size="sm"
        onClick={onZoomIn}
        aria-label="Zoom in"
      >
        <Icon name="zoom-in" size="sm" />
      </Button>
      
      <div className="w-px h-6 bg-gray-300 mx-1" />
      
      <Button
        variant="outline"
        size="sm" 
        onClick={onReset}
        aria-label="Reset zoom"
      >
        <Icon name="reset" size="sm" />
      </Button>
      
      <Button
        variant="outline"
        size="sm"
        onClick={onFit}
        aria-label="Fit to screen"
      >
        <Icon name="fit" size="sm" />
      </Button>
    </div>
  );
};