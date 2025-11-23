import { useState } from 'react'
import type { MemoNode } from './types/memo.types'
import { ZoomControls } from './components/molecules/ZoomControls'
import { ModeToggle } from './components/molecules/ModeToggle'
import { NodeTitle } from './components/molecules/NodeTitle'

// Mock data for testing
const mockData: MemoNode[] = [
  {
    id: 'main-1',
    title: 'メイン関数',
    level: 1,
    description: 'アプリケーションのエントリーポイントです',
    path: 'src/main.rs:10',
    content: 'メイン関数の説明...',
    codeBlocks: [],
    children: [
      {
        id: 'config-2',
        title: '設定読み込み',
        level: 2,
        description: '設定ファイルを読み込んで初期化する関数',
        path: 'src/config.rs:25',
        content: '設定の説明...',
        codeBlocks: [],
        children: []
      }
    ]
  }
]

function App() {
  const [memoData] = useState<MemoNode[]>(mockData)
  const [currentMode, setCurrentMode] = useState<'memo' | 'flow'>('memo')
  const [zoom, setZoom] = useState(1.0)

  const handleZoomIn = () => setZoom(prev => Math.min(prev * 1.25, 5.0))
  const handleZoomOut = () => setZoom(prev => Math.max(prev * 0.8, 0.1))
  const handleResetZoom = () => setZoom(1.0)
  const handleFitZoom = () => setZoom(0.8) // Mock implementation

  return (
    <div className="font-mono p-5 bg-gray-100 min-h-screen">
      {/* Controls */}
      <div className="fixed top-5 left-5 flex gap-2.5 z-50">
        <ZoomControls
          zoom={zoom}
          onZoomIn={handleZoomIn}
          onZoomOut={handleZoomOut}
          onReset={handleResetZoom}
          onFit={handleFitZoom}
        />
        <ModeToggle
          currentMode={currentMode}
          onModeChange={setCurrentMode}
        />
      </div>

      {/* Content */}
      <div 
        className="mt-20 transition-transform duration-200 ease-out origin-top-left"
        style={{ transform: `scale(${zoom})` }}
      >
        <h1 className="mb-5 text-gray-800">
          Current Mode: {currentMode}
        </h1>
        
        {memoData.map(node => (
          <div key={node.id} className="border-4 border-gray-800 rounded-lg bg-white m-4 p-2.5">
            <NodeTitle
              title={node.title}
              level={node.level}
              description={node.description}
              path={node.path}
              hasChildren={node.children.length > 0}
            />
            {node.children.map(child => (
              <div key={child.id} className="ml-5 mt-2.5">
                <NodeTitle
                  title={child.title}
                  level={child.level}
                  description={child.description}
                  path={child.path}
                />
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  )
}

export default App
