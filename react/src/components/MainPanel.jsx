
import React, { useState } from 'react';
import { useNotes } from '../contexts/NotesContext';
import { useUI } from '../contexts/UIContext';
import EditorPanel from './EditorPanel';
import RelatedNotes from './RelatedNotes';
import GraphView from './GraphView';
import { useTranslation } from 'react-i18next';

export default function MainPanel({ editor }) {
  const { t } = useTranslation();
  const { postId, title, onTitleChange, relatedPosts, loadNode } = useNotes();
  const { isSidebarHidden, setIsSidebarHidden, showGraphView, setShowGraphView } = useUI();

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', position: 'relative' }}>
      <div style={{ marginBottom: 10, display: 'flex', justifyContent: 'flex-end', gap: 10 }}>
        <button onClick={() => setShowGraphView(false)} style={{ padding: '4px 12px', borderRadius: 6 }}>
          {t('editor_view')}
        </button>
        <button onClick={() => setShowGraphView(true)} style={{ padding: '4px 12px', borderRadius: 6 }}>
          {t('graph_view')}
        </button>
      </div>

      {showGraphView ? (
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
          {t('show_sidebar')}
        </button>
      }
    </div>
  );
}
