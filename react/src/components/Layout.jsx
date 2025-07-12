
import React, { useState } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { useUI } from '../contexts/UIContext';
import Sidebar from './Sidebar';
import MainPanel from './MainPanel';
import LoginModal from './LoginModal';
import SignupModal from './SignupModal';



import { useNotes } from '../contexts/NotesContext';

export default function Layout({ editor }) {
  const { 
    isLoggedIn, handleLogin, handleSignup, 
    loginError, setLoginError, signupError, setSignupError, 
    signupSuccess, setSignupSuccess 
  } = useAuth();
  
  const { 
    isLoginModalOpen, setIsLoginModalOpen, 
    isSignupModalOpen, setIsSignupModalOpen, 
    isSidebarHidden, handleMouseDown 
  } = useUI();

  const { log, isSaving } = useNotes();

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

      <pre style={{
        position: 'fixed',
        opacity: 0.8,
        top: 20,
        right: 20,
        width: 350,
        height: 200,
        background: 'rgba(0, 0, 0, 0.7)',
        color: '#0f0',
        padding: 10,
        overflowY: 'auto',
        borderRadius: 8,
        zIndex: 2000,
        fontSize: 12,
        border: '1px solid #444',
        backdropFilter: 'blur(5px)',
        boxShadow: '0 4px 12px rgba(0,0,0,0.5)'
      }}>{isSaving ? "⏳ 자동저장 중...\n" : ""}{log}</pre>

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
