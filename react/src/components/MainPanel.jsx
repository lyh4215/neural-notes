
import React, { useState } from 'react';
import { useNotes } from '../contexts/NotesContext';
import { useUI } from '../contexts/UIContext';
import EditorPanel from './EditorPanel';
import RelatedNotes from './RelatedNotes';
import GraphView from './GraphView';

export default function MainPanel({ editor }) {
  const { postId, title, onTitleChange, relatedPosts, loadNode } = useNotes();
  const { isSidebarHidden, setIsSidebarHidden } = useUI();
  const [showGraph, setShowGraph] = useState(false);

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', position: 'relative' }}>
      <div style={{ marginBottom: 10, display: 'flex', justifyContent: 'flex-end', gap: 10 }}>
        <button onClick={() => setShowGraph(false)} style={{ padding: '4px 12px', borderRadius: 6 }}>
          에디터 뷰
        </button>
        <button onClick={() => setShowGraph(true)} style={{ padding: '4px 12px', borderRadius: 6 }}>
          그래프 뷰
        </button>
      </div>

      {showGraph ? (
        <GraphView />
      ) : (
        <>
          <EditorPanel
            editor={editor}
            postId={postId}
            title={title}
            onTitleChange={onTitleChange}
          />
          <RelatedNotes
            relatedPosts={relatedPosts}
            loadNode={loadNode}
          />
        </>
      )}

      {isSidebarHidden &&
        <button
          onClick={() => setIsSidebarHidden(false)}
          style={{
            position: 'fixed',
            left: 20,
            bottom: 20,
            zIndex: 101,
            padding: '8px 18px',
            background: '#222',
            color: '#fff',
            border: '1px solid #555',
            borderRadius: 8,
            fontSize: 14,
            cursor: 'pointer',
            opacity: 0.85
          }}
        >
          ➡ 사이드바 나타내기
        </button>
      }
    </div>
  );
}
