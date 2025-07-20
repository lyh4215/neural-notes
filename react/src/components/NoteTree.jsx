// src/components/NoteTree.jsx
import React from 'react';
import formatTime from '../utils/formatTime';

import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

export default function NoteTree({
  treeData, loadNode, handleDelete, isLoggedIn, showDeleteFor, setShowDeleteFor
}) {
  const [expanded, setExpanded] = useState({});
  const navigate = useNavigate();

  const toggleExpand = (path) => {
    setExpanded(prev => ({ ...prev, [path]: !prev[path] }));
  };

  const handleNodeClick = (node) => {
    if (node.postId) {
      navigate(`/posts/${node.postId}`);
    }
  };
  const renderTree = (nodes, parentPath = '', depth = 0) =>
    nodes?.map(node => {
      const path = parentPath ? `${parentPath}/${node.name}` : node.name;
      const hasChildren = node.children?.length > 0;
      const isExpanded = expanded[path];
      const showMenuBtn = node.postId && isLoggedIn;
      const key = node.postId ? `${path}-${node.postId}` : path;

      return (
        <div key={key}>
          <div style={{ display: 'flex', alignItems: 'center', padding: '4px 0', marginLeft: depth * 16 }}>
            {hasChildren && (
              <span
                onClick={e => { e.stopPropagation(); toggleExpand(path); }}
                style={{
                  marginRight: 4, color: '#ccc', cursor: 'pointer', background: '#444', borderRadius: 4,
                  width: 20, height: 20, display: 'inline-flex', alignItems: 'center', justifyContent: 'center', fontSize: 12
                }}>
                {isExpanded ? 'v' : '>'}
              </span>
            )}
            <span
              onClick={() => handleNodeClick(node)}
              style={{ color: '#fff', cursor: node.postId ? 'pointer' : 'default', paddingLeft: hasChildren ? 4 : 24 }}
            >
              {node.name}
            </span>
            {node.postId && (
              <>
                <span style={{ marginLeft: 8, fontSize: 12, color: '#888' }}>{formatTime(node.updatedAt)}</span>
                {showMenuBtn &&
                  <div style={{ position: 'relative', marginLeft: 6 }}>
                    <button
                      style={{
                        background: '#333', border: 'none', borderRadius: 4, color: '#fff', width: 22, height: 22,
                        cursor: 'pointer', padding: 0, fontSize: 18, lineHeight: '22px'
                      }}
                      onClick={e => { e.stopPropagation(); setShowDeleteFor(showDeleteFor === node.postId ? null : node.postId); }}
                      tabIndex={-1}
                    >⋯</button>
                    {showDeleteFor === node.postId &&
                      <div style={{
                        position: 'absolute', top: 24, right: 0, background: '#222', border: '1px solid #444',
                        borderRadius: 6, boxShadow: '0 2px 12px #000a', zIndex: 100, minWidth: 80
                      }}>
                        <button
                          onClick={e => { e.stopPropagation(); handleDelete(node.postId); setShowDeleteFor(null); }}
                          style={{
                            width: '100%', padding: '8px 0', color: '#f44', background: 'none',
                            border: 'none', borderRadius: 6, fontWeight: 600, cursor: 'pointer'
                          }}
                        >삭제</button>
                      </div>
                    }
                  </div>
                }
              </>
            )}
          </div>
          {hasChildren && isExpanded && renderTree(node.children, path, depth + 1)}
        </div>
      );
    });

  return <>{renderTree(treeData)}</>;
}
