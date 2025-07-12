
import React from 'react';
import { useAuth } from '../contexts/AuthContext';
import { useNotes } from '../contexts/NotesContext';
import { useUI } from '../contexts/UIContext';
import NoteTree from './NoteTree';

export default function Sidebar() {
  const { isLoggedIn, loggedInUsername, handleLogout } = useAuth();
  const { searchKeyword, setSearchKeyword, treeData, loadNode, handleDelete, handleNew, isSaving, log } = useNotes();
  const { setIsLoginModalOpen, setIsSignupModalOpen, isSidebarHidden, setIsSidebarHidden, showDeleteFor, setShowDeleteFor, dividerPosition } = useUI();

  if (isSidebarHidden) return null;

  return (
    <div
      style={{
        width: dividerPosition,
        minWidth: '30vw',
        maxWidth: '90vw',
        paddingRight: 20,
        display: 'flex',
        flexDirection: 'column',
        position: 'relative',
        transition: 'width 0.18s'
      }}
    >
      <div style={{ marginBottom: 10, width: '100%', display: 'flex', alignItems: 'center', gap: 10 }}>
        {isLoggedIn ? (
          <>
            <span style={{ color: '#fff', fontWeight: 600, fontSize: 14 }}>{loggedInUsername}</span>
            <button onClick={handleLogout} style={{ padding: '4px 12px', borderRadius: 6 }}>로그아웃</button>
          </>
        ) : (
          <>
            <button onClick={() => setIsLoginModalOpen(true)} style={{ padding: '4px 12px', borderRadius: 6 }}>로그인</button>
            <button onClick={() => setIsSignupModalOpen(true)} style={{ padding: '4px 12px', borderRadius: 6 }}>회원가입</button>
          </>
        )}
      </div>
      <input type="text" placeholder="검색어 입력..." value={searchKeyword} onChange={e => setSearchKeyword(e.target.value)}
        style={{
          marginBottom: 10, width: '100%', padding: 8, borderRadius: 4,
          border: '1px solid #444', background: '#2e2e2e', color: '#fff'
        }} />
      <h2 style={{ color: '#fff', margin: 0, marginBottom: 10 }}>🧠 Neural Notes</h2>
      <div style={{ marginBottom: 10, display: 'flex', gap: 10, flexWrap: 'wrap' }}>
        <button onClick={handleNew} disabled={!isLoggedIn}>🆕 New</button>
      </div>
      <div style={{ flex: 1, overflowY: 'auto', padding: 10, background: '#1e1e1e' }}>
        <NoteTree
          treeData={treeData}
          loadNode={loadNode}
          handleDelete={handleDelete}
          isLoggedIn={isLoggedIn}
          showDeleteFor={showDeleteFor}
          setShowDeleteFor={setShowDeleteFor}
        />
      </div>
      <pre style={{
        background: "#111", color: "#0f0", padding: 10,
        marginTop: 10, height: 150, overflowY: "auto", borderRadius: 4
      }}>{isSaving ? "⏳ 자동저장 중...\n" : ""}{log}</pre>
      <button
        onClick={() => setIsSidebarHidden(true)}
        style={{
          position: 'absolute',
          left: 20,
          bottom: 20,
          zIndex: 100,
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
        ⬅ 사이드바 숨기기
      </button>
    </div>
  );
}
