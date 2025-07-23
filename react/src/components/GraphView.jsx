import React, { useState, useEffect, useRef, useCallback } from 'react';
import { ForceGraph2D } from 'react-force-graph';
import { useUI } from '../contexts/UIContext';
import { useAuth } from '../contexts/AuthContext';
import { useNotes } from '../contexts/NotesContext';
import api from '../api';
import { useTranslation } from 'react-i18next';

export default function GraphView() {
  const { t } = useTranslation();
  const [graphData, setGraphData] = useState({ nodes: [], links: [] });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const { isLoggedIn } = useAuth();
  const { focusedNodeId, setFocusedNodeId, setShowGraphView } = useUI();
  const { loadNode } = useNotes();
  const fgRef = useRef();

  const [showContextMenu, setShowContextMenu] = useState(false);
  const [contextMenuPos, setContextMenuPos] = useState({ x: 0, y: 0 });
  const [contextMenuNodeId, setContextMenuNodeId] = useState(null);
  const [selectedNodeId, setSelectedNodeId] = useState(null);

  const fetchGraphData = useCallback(async () => {
    if (!isLoggedIn) {
      setError(t('login_required'));
      setLoading(false);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const response = await api.get('/posts/graph');
      setGraphData(response.data);
    } catch (err) {
      console.error('Failed to fetch graph data:', err);
      setError(t('graph_error'));
    } finally {
      setLoading(false);
    }
  }, [isLoggedIn, t]);

  useEffect(() => {
    fetchGraphData();
  }, [fetchGraphData]);

  useEffect(() => {
    if (fgRef.current && focusedNodeId) {
      const node = graphData.nodes.find(n => n.id === String(focusedNodeId));
      if (node) {
        fgRef.current.centerAt(node.x, node.y, 1000); // 1초 동안 이동
        fgRef.current.zoom(2.5, 1000); // 1초 동안 확대
      }
      setFocusedNodeId(null); // 포커스가 끝나면 ID를 리셋
    }
  }, [focusedNodeId, graphData.nodes, setFocusedNodeId]);

  useEffect(() => {
    const fg = fgRef.current;
    if (fg && !loading && !error && graphData.nodes.length > 0) {
      fg.d3Force('charge').strength(-200);
      fg.d3Force('link').distance(link => 150 * (1 - link.value));

      const timer = setTimeout(() => {
        fg.zoomToFit(400);
      }, 150);

      return () => clearTimeout(timer);
    }
  }, [loading, error, graphData]);

  const handleNodeClick = useCallback(node => {
    const fg = fgRef.current;
    if (!fg) return;

    const screenCoords = fg.graph2ScreenCoords(node.x, node.y);

    setShowContextMenu(true);
    setContextMenuPos({ x: screenCoords.x, y: screenCoords.y + 40 });
    setContextMenuNodeId(node.id);
    setSelectedNodeId(node.id);
  }, []);

  const handleBackgroundClick = useCallback(() => {
    setShowContextMenu(false);
    setSelectedNodeId(null);
  }, []);

  if (loading) return <div style={{ color: '#fff' }}>{t('graph_loading')}</div>;
  if (error) return <div style={{ color: '#f44' }}>{error}</div>;
  if (graphData.nodes.length === 0) return <div style={{ color: '#fff' }}>{t('no_graph_data')}</div>;

  return (
    <div style={{
      flex: 1,
      display: 'flex',
      flexDirection: 'column',
      borderRadius: 8,
      overflow: 'hidden',
      backgroundColor: '#1a1a1a'
    }}>
      <h2 style={{ color: '#fff', margin: 0, padding: 10, textAlign: 'center', background: '#1a1a1a' }}>{t('similar_notes_graph')}</h2>
      <ForceGraph2D
        ref={fgRef}
        graphData={graphData}
        nodeId="id"
        nodeLabel="name"
        linkSource="source"
        linkTarget="target"
        linkColor={link => {
          const isSelected = selectedNodeId && (link.source.id === selectedNodeId || link.target.id === selectedNodeId);
          return isSelected ? 'rgba(255, 215, 0, 0.8)' : '#555';
        }}
        linkWidth={link => {
          const isSelected = selectedNodeId && (link.source.id === selectedNodeId || link.target.id === selectedNodeId);
          return (isSelected ? 2 : 1) * link.value * 3;
        }}
        onBackgroundPaint={(ctx) => {
          if (!fgRef.current) return;

          const gridSize = 40;
          const gridColor = '#2c2c2c';
          const transform = fgRef.current.getTransform();

          // Get the visible area in graph coordinates
          const topLeft = fgRef.current.screen2GraphCoords(0, 0);
          const bottomRight = fgRef.current.screen2GraphCoords(ctx.canvas.width, ctx.canvas.height);

          ctx.strokeStyle = gridColor;
          ctx.lineWidth = 1 / transform.k; // Keep line width consistent regardless of zoom

          // Draw vertical lines
          for (let x = Math.floor(topLeft.x / gridSize) * gridSize; x < bottomRight.x; x += gridSize) {
            ctx.beginPath();
            ctx.moveTo(x, topLeft.y);
            ctx.lineTo(x, bottomRight.y);
            ctx.stroke();
          }

          // Draw horizontal lines
          for (let y = Math.floor(topLeft.y / gridSize) * gridSize; y < bottomRight.y; y += gridSize) {
            ctx.beginPath();
            ctx.moveTo(topLeft.x, y);
            ctx.lineTo(bottomRight.x, y);
            ctx.stroke();
          }
        }}
        nodeAutoColorBy="name" // 노드 색상을 이름에 따라 자동 지정
        nodeCanvasObject={(node, ctx, globalScale) => {
          const isSelected = selectedNodeId === node.id;
          const FIXED_NODE_WIDTH = 80;
          const FIXED_NODE_HEIGHT = 24;
          const FIXED_FONT_SIZE = 14;
          const FIXED_CORNER_RADIUS = 4;

          const nodeWidth = FIXED_NODE_WIDTH / globalScale;
          const nodeHeight = FIXED_NODE_HEIGHT / globalScale;
          const cornerRadius = FIXED_CORNER_RADIUS / globalScale;
          const fontSize = FIXED_FONT_SIZE / globalScale;

          // Truncate text
          let label = node.name;
          ctx.font = `600 ${fontSize}px Sans-Serif`;
          let textWidth = ctx.measureText(label).width;
          // Use FIXED_NODE_WIDTH for truncation logic
          while (textWidth > FIXED_NODE_WIDTH - 10 && label.length > 1) {
            label = label.slice(0, -1);
            textWidth = ctx.measureText(label + '...').width;
          }
          if (label !== node.name) {
            label += '...';
          }

          // Draw rounded rectangle
          ctx.beginPath();
          ctx.moveTo(node.x - nodeWidth / 2 + cornerRadius, node.y - nodeHeight / 2);
          ctx.arcTo(node.x + nodeWidth / 2, node.y - nodeHeight / 2, node.x + nodeWidth / 2, node.y + nodeHeight / 2, cornerRadius);
          ctx.arcTo(node.x + nodeWidth / 2, node.y + nodeHeight / 2, node.x - nodeWidth / 2, node.y + nodeHeight / 2, cornerRadius);
          ctx.arcTo(node.x - nodeWidth / 2, node.y + nodeHeight / 2, node.x - nodeWidth / 2, node.y - nodeHeight / 2, cornerRadius);
          ctx.arcTo(node.x - nodeWidth / 2, node.y - nodeHeight / 2, node.x + nodeWidth / 2, node.y - nodeHeight / 2, cornerRadius);
          ctx.closePath();

          // Style and fill
          ctx.fillStyle = isSelected ? 'rgba(255, 215, 0, 0.9)' : 'rgba(0, 0, 0, 0.7)';
          ctx.fill();

          // Text
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillStyle = isSelected ? '#111' : '#fff';
          ctx.fillText(label, node.x, node.y);

          node.__bckgDimensions = [nodeWidth, nodeHeight];
        }}
        nodePointerAreaPaint={(node, color, ctx) => {
          if (!node.__bckgDimensions) return;
          const [width, height] = node.__bckgDimensions;
          ctx.fillStyle = color;
          ctx.fillRect(node.x - width / 2, node.y - height / 2, width, height);
        }}
        onNodeClick={handleNodeClick}
        onBackgroundClick={handleBackgroundClick}
        width={window.innerWidth * 0.6 - 40} // MainPanel의 대략적인 너비에 맞춤
        height={window.innerHeight * 0.8} // 적절한 높이 설정
      />
      {showContextMenu && (
        <div
          style={{
            position: 'absolute',
            left: contextMenuPos.x,
            top: contextMenuPos.y,
            background: '#333',
            border: '1px solid #555',
            borderRadius: 4,
            padding: 5,
            zIndex: 1000,
          }}
        >
          <button
            onClick={() => {
              loadNode({ postId: contextMenuNodeId });
              setShowGraphView(false);
              setShowContextMenu(false);
              setSelectedNodeId(null);
            }}
            style={{
              background: 'none',
              border: 'none',
              color: '#fff',
              padding: '5px 10px',
              cursor: 'pointer',
              width: '100%',
              textAlign: 'left',
            }}
          >
            {t('open')}
          </button>
        </div>
      )}
    </div>
  );
}
