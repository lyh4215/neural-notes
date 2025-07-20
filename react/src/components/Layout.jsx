
import React, { useState, useEffect } from 'react';
import { useLocation, useParams } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { useNotes } from '../contexts/NotesContext';
import { useUI } from '../contexts/UIContext';
import Sidebar from './Sidebar';
import MainPanel from './MainPanel';
import LoginModal from './LoginModal';
import SignupModal from './SignupModal';

export default function Layout({ editor }) {
  const { 
    handleLogin, handleSignup, 
    loginError, signupError, signupSuccess 
  } = useAuth();
  
  const { 
    isLoginModalOpen, setIsLoginModalOpen, 
    isSignupModalOpen, setIsSignupModalOpen, 
    isSidebarHidden, handleMouseDown, 
    isLogPanelVisible, setIsLogPanelVisible
  } = useUI();

  const { 
    loadNode, setPostId, setTitle, setRelatedPosts, 
    isSilentUpdate, log, isSaving 
  } = useNotes();

  const { id } = useParams();
  const location = useLocation();

  const [loginUsername, setLoginUsername] = useState('');
  const [loginPassword, setLoginPassword] = useState('');
  const [signupUsername, setSignupUsername] = useState('');
  const [signupPassword, setSignupPassword] = useState('');

  const onLoginSubmit = async (e) => {
    e.preventDefault();
    const success = await handleLogin(loginUsername, loginPassword);
    if (success) {
      setIsLoginModalOpen(false);
      setLoginUsername('');
      setLoginPassword('');
    }
  };

  const onSignupSubmit = async (e) => {
    e.preventDefault();
    const success = await handleSignup(signupUsername, signupPassword);
    if (success) {
      setSignupUsername('');
      setSignupPassword('');
    }
  };

  // URL 변경 감지 및 노트 로드/초기화
  useEffect(() => {
    console.log('Layout useEffect triggered. id:', id, 'editor:', editor);
    if (!editor) {
      console.log('Editor not ready in Layout useEffect.');
      return; // editor가 준비되지 않았으면 아무것도 하지 않음
    }

    if (id) {
      console.log('Loading note with ID:', id);
      if (!isSilentUpdate.current) {
        loadNode({ postId: id });
      } else {
        console.log('isSilentUpdate is true, skipping loadNode.');
      }
    } else {
      console.log('No ID in URL, resetting editor.');
      // URL이 '/'일 때 상태 초기화
      setPostId("");
      setTitle("");
      editor.commands.setContent('<p>✍️ 여기서 글을 작성하세요</p>');
      setRelatedPosts([]);
    }
  }, [id, editor]); // id가 바뀔 때마다 실행

  return (
    <div style={{
      display: 'flex', padding: 20, fontFamily: 'Arial', height: '100vh',
      width: '100vw', boxSizing: 'border-box', background: '#1e1e1e', userSelect: 'none',
      position: 'relative'
    }}>
      <Sidebar />

      {!isSidebarHidden &&
        <div
          onMouseDown={handleMouseDown}
          style={{
            width: 5,
            margin: '0 10px',
            cursor: 'col-resize',
            background: '#333',
            userSelect: 'none',
            zIndex: 100,
          }}
        />
      }

      <MainPanel editor={editor} />

      

      <LoginModal
        isOpen={isLoginModalOpen}
        onClose={() => setIsLoginModalOpen(false)}
        onSubmit={onLoginSubmit}
        username={loginUsername}
        setUsername={setLoginUsername}
        password={loginPassword}
        setPassword={setLoginPassword}
        error={loginError}
      />
      <SignupModal
        isOpen={isSignupModalOpen}
        onClose={() => setIsSignupModalOpen(false)}
        onSubmit={onSignupSubmit}
        username={signupUsername}
        setUsername={setSignupUsername}
        password={signupPassword}
        setPassword={setSignupPassword}
        error={signupError}
        success={signupSuccess}
      />
    </div>


  );
}
