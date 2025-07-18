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
      const response = await api.get('http://localhost:3000/posts/graph');
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
      fg.d3Force('charge').strength(-120);
      fg.d3Force('link').distance(link => 100 * (1 - link.value));

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
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', background: '#1a1a1a', borderRadius: 8, overflow: 'hidden' }}>
      <h2 style={{ color: '#fff', margin: 0, padding: 10, textAlign: 'center' }}>유사 노트 그래프</h2>
      <ForceGraph2D
        ref={fgRef}
        graphData={graphData}
        nodeId="id"
        nodeLabel="name"
        linkSource="source"
        linkTarget="target"
        linkWidth={link => link.value * 5} // 유사도에 따라 링크 두께 조절
        linkDirectionalArrowLength={3} // 링크 방향 화살표 길이
        linkDirectionalArrowRelPos={1} // 링크 방향 화살표 위치
        nodeAutoColorBy="name" // 노드 색상을 이름에 따라 자동 지정
        nodeCanvasObject={(node, ctx, globalScale) => {
          const label = node.name;
          const fontSize = 12 / globalScale;
          ctx.font = `${fontSize}px Sans-Serif`;
          const textWidth = ctx.measureText(label).width;
          const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2); // some padding

          ctx.fillStyle = '#333'; // 노드 배경색
          ctx.fillRect(node.x - bckgDimensions[0] / 2, node.y - bckgDimensions[1] / 2, bckgDimensions[0], bckgDimensions[1]);

          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillStyle = '#fff'; // 노드 텍스트 색상
          ctx.fillText(label, node.x, node.y);

          node.__bckgDimensions = bckgDimensions; // for hit testing
        }}
        nodePointerAreaPaint={(node, color, ctx) => {
          ctx.fillStyle = color;
          const bckgDimensions = node.__bckgDimensions;
          bckgDimensions && ctx.fillRect(node.x - bckgDimensions[0] / 2, node.y - bckgDimensions[1] / 2, bckgDimensions[0], bckgDimensions[1]);
        }}
        onNodeClick={handleNodeClick}
        width={window.innerWidth * 0.6 - 40} // MainPanel의 대략적인 너비에 맞춤
        height={window.innerHeight * 0.8} // 적절한 높이 설정
      />
    </div>
  );
}