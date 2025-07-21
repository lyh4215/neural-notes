// src/components/EditorPanel.jsx
import React from 'react';
import { EditorContent } from '@tiptap/react';
import '../tiptap.css';
import { useTranslation } from 'react-i18next';

export default function EditorPanel({ editor, postId, title, onTitleChange }) {
  const { t } = useTranslation();
  return (
    <div style={{
      flex: 1, // 👈 부모 자체가 flex 아이템이 되어야 함!
      display: 'flex',
      flexDirection: 'column',
      minHeight: 0,
      minWidth: 0,
    }}>
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: 10 }}>
        <h2 style={{ color: '#fff', margin: 0 }}>{t('editor')}</h2>
        <input
          type="text"
          placeholder={t('title')}
          value={title}
          onChange={onTitleChange}
          disabled={!postId}
          style={{
            flex: 1, marginLeft: 20, padding: 8, fontSize: 16,
            borderRadius: 4, border: '1px solid #444', background: '#2e2e2e',
            color: '#fff', opacity: !postId ? 0.6 : 1
          }}
        />
      </div>
      <div
        style={{
          position: 'relative',
          flex: 1,                // 🟢 핵심: 세로 남는 공간 모두 차지
          minHeight: 0,
          minWidth: 0,
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
        }}
      >
        <EditorContent
          editor={editor}
          style={{
            flex: 1,            // 🟢 핵심: 세로로 꽉!
            minHeight: 0,
            minWidth: 0,
            border: 'none',
            outline: 'none',
            background: !postId ? "#222" : undefined,
            borderRadius: 8,
            filter: !postId ? "grayscale(0.7) brightness(0.8)" : "none",
            overflowY: 'auto',
            overflowX: 'hidden',
            wordBreak: 'break-all',
            whiteSpace: 'pre-wrap',
            height: '100%',     // 🟢 혹시 모르니 height도 추가
          }}
        />
        {!postId &&
          <div style={{
            position: 'absolute', top: 0, left: 0, right: 0, bottom: 0,
            background: 'rgba(30,30,30,0.6)', zIndex: 1,
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            color: '#aaa', fontSize: 20, fontWeight: 600, pointerEvents: 'all',
            borderRadius: 8, userSelect: 'none',
          }}>
            {t('no_note_selected')}
          </div>
        }
      </div>
    </div>
  );
}
