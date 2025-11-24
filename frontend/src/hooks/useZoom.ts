import { useState, useCallback } from 'react';
import type { ZoomState } from '../types';

export const useZoom = () => {
  const [zoomState, setZoomState] = useState<ZoomState>({
    zoom: 1.0,
    panX: 0,
    panY: 0
  });

  const zoomIn = useCallback(() => {
    setZoomState(prev => ({
      ...prev,
      zoom: Math.min(prev.zoom * 1.25, 5.0)
    }));
  }, []);

  const zoomOut = useCallback(() => {
    setZoomState(prev => ({
      ...prev,
      zoom: Math.max(prev.zoom * 0.8, 0.1)
    }));
  }, []);

  const resetZoom = useCallback(() => {
    setZoomState({
      zoom: 1.0,
      panX: 0,
      panY: 0
    });
  }, []);

  const fitToScreen = useCallback(() => {
    setZoomState(prev => ({
      zoom: 1.0,
      panX: 0,
      panY: 0
    }));
  }, []);

  const updatePan = useCallback((deltaX: number, deltaY: number) => {
    setZoomState(prev => ({
      ...prev,
      panX: prev.panX + deltaX,
      panY: prev.panY + deltaY
    }));
  }, []);

  const zoomAtPoint = useCallback((factor: number, mouseX: number, mouseY: number) => {
    setZoomState(prev => {
      const newZoom = Math.max(Math.min(prev.zoom * factor, 5.0), 0.1);
      
      // Calculate the point under the mouse in the original coordinate system
      const pointX = (mouseX - prev.panX) / prev.zoom;
      const pointY = (mouseY - prev.panY) / prev.zoom;
      
      // Adjust pan so the point under the mouse stays in the same place
      const newPanX = mouseX - pointX * newZoom;
      const newPanY = mouseY - pointY * newZoom;
      
      return {
        zoom: newZoom,
        panX: newPanX,
        panY: newPanY
      };
    });
  }, []);

  return {
    zoomState,
    zoomIn,
    zoomOut,
    resetZoom,
    fitToScreen,
    updatePan,
    zoomAtPoint
  };
};