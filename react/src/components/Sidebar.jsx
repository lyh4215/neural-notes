import React from 'react';
import { useAuth } from '../contexts/AuthContext';
import { useNotes } from '../contexts/NotesContext';
import { useUI } from '../contexts/UIContext';
import NoteTree from './NoteTree';
import { useTranslation } from 'react-i18next';

export default function Sidebar() {
  const { t, i18n } = useTranslation();
  const { isLoggedIn, loggedInUsername, handleLogout } = useAuth();
  const { 
    searchKeyword, setSearchKeyword, treeData, loadNode, handleDelete, handleNew, 
    isLoadingList, listError, handleSearch, isSearchMode, handleListLoad 
  } = useNotes();
  const { setIsLoginModalOpen, setIsSignupModalOpen, isSidebarHidden, setIsSidebarHidden, showDeleteFor, setShowDeleteFor, dividerPosition } = useUI();

  if (isSidebarHidden) return null;

  const onSearchKeyDown = (e) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

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
            <button onClick={handleLogout} style={{ padding: '4px 12px', borderRadius: 6 }}>{t('logout')}</button>
          </>
        ) : (
          <>
            <button onClick={() => setIsLoginModalOpen(true)} style={{ padding: '4px 12px', borderRadius: 6 }}>{t('login')}</button>
            <button onClick={() => setIsSignupModalOpen(true)} style={{ padding: '4px 12px', borderRadius: 6 }}>{t('signup')}</button>
          </>
        )}
      </div>
      <div style={{ display: 'flex', gap: 10, marginBottom: 10 }}>
        <input 
          type="text" 
          placeholder={t('search_placeholder')} 
          value={searchKeyword} 
          onChange={e => setSearchKeyword(e.target.value)}
          onKeyDown={onSearchKeyDown}
          style={{
            flex: 1, padding: 8, borderRadius: 4,
            border: '1px solid #444', background: '#2e2e2e', color: '#fff'
          }} 
        />
        <button onClick={handleSearch} disabled={!isLoggedIn || isLoadingList}>{t('search')}</button>
      </div>
      <h2 style={{ color: '#fff', margin: 0, marginBottom: 10 }}>{t('neural_notes')}</h2>
      <div style={{ marginBottom: 10, display: 'flex', gap: 10, flexWrap: 'wrap' }}>
        <button onClick={handleNew} disabled={!isLoggedIn}>{t('new_note')}</button>
        {isSearchMode && <button onClick={handleListLoad}>{t('show_all_notes')}</button>}
      </div>
      <div style={{ flex: 1, overflowY: 'auto', padding: 10, background: '#1e1e1e' }}>
        {isLoadingList ? (
          <div style={{ color: '#888', textAlign: 'center', marginTop: 20 }}>{t('loading_notes')}</div>
        ) : listError ? (
          <div style={{ color: '#f44', textAlign: 'center', marginTop: 20 }}>{listError}</div>
        ) : (
          <NoteTree
            treeData={treeData}
            loadNode={loadNode}
            handleDelete={handleDelete}
            isLoggedIn={isLoggedIn}
            showDeleteFor={showDeleteFor}
            setShowDeleteFor={setShowDeleteFor}
          />
        )}
      </div>
      
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
        {t('hide_sidebar')}
      </button>
      <div style={{ position: 'absolute', bottom: 20, right: 20, display: 'flex', gap: 5 }}>
        <button onClick={() => i18n.changeLanguage('ko')} style={{ padding: '4px 8px', borderRadius: 4, fontSize: 12, background: i18n.language === 'ko' ? '#646cff' : '#1a1a1a' }}>KO</button>
        <button onClick={() => i18n.changeLanguage('en')} style={{ padding: '4px 8px', borderRadius: 4, fontSize: 12, background: i18n.language === 'en' ? '#646cff' : '#1a1a1a' }}>EN</button>
      </div>
    </div>
  );
}