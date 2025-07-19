import React, { useState, useEffect, useRef, useCallback } from 'react';
import { ForceGraph2D } from 'react-force-graph';
import { useAuth } from '../contexts/AuthContext';
import api from '../api';

export default function GraphView() {
  const [graphData, setGraphData] = useState({ nodes: [], links: [] });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const { isLoggedIn } = useAuth();
  const fgRef = useRef();

  const fetchGraphData = useCallback(async () => {
    if (!isLoggedIn) {
      setError('로그인이 필요합니다.');
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
      setError('그래프 데이터를 불러오는데 실패했습니다.');
    } finally {
      setLoading(false);
    }
  }, [isLoggedIn]);

  useEffect(() => {
    fetchGraphData();
  }, [fetchGraphData]);

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
    // 노드 클릭 시 해당 게시물로 이동 (필요하다면)
    // window.location.href = `/posts/${node.id}`;
    console.log('Node clicked:', node.id, node.name);
  }, []);

  if (loading) return <div style={{ color: '#fff' }}>그래프 로딩 중...</div>;
  if (error) return <div style={{ color: '#f44' }}>{error}</div>;
  if (graphData.nodes.length === 0) return <div style={{ color: '#fff' }}>표시할 노트가 없습니다.</div>;

  return (
    <div style={{
      flex: 1, 
      display: 'flex', 
      flexDirection: 'column', 
      borderRadius: 8, 
      overflow: 'hidden',
      backgroundColor: '#1a1a1a'
    }}>
      <h2 style={{ color: '#fff', margin: 0, padding: 10, textAlign: 'center', background: '#1a1a1a' }}>유사 노트 그래프</h2>
      <ForceGraph2D
        ref={fgRef}
        graphData={graphData}
        nodeId="id"
        nodeLabel="name"
        linkSource="source"
        linkTarget="target"
        linkWidth={link => link.value * 5} // 유사도에 따라 링크 두께 조절
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
          ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
          ctx.fill();

          // Text
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillStyle = '#fff';
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
        width={window.innerWidth * 0.6 - 40} // MainPanel의 대략적인 너비에 맞춤
        height={window.innerHeight * 0.8} // 적절한 높이 설정
      />
    </div>
  );
}