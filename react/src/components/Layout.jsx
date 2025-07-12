
import React, { useState } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { useUI } from '../contexts/UIContext';
import Sidebar from './Sidebar';
import MainPanel from './MainPanel';
import LoginModal from './LoginModal';
import SignupModal from './SignupModal';

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
