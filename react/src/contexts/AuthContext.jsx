
import React, { createContext, useState, useContext, useCallback } from 'react';
import api from '../api';

const AuthContext = createContext();



export const AuthProvider = ({ children }) => {
  const [isLoggedIn, setIsLoggedIn] = useState(!!localStorage.getItem('accessToken'));
  const [loggedInUsername, setLoggedInUsername] = useState(localStorage.getItem('username') || '');
  const [loginError, setLoginError] = useState('');
  const [signupError, setSignupError] = useState('');
  const [signupSuccess, setSignupSuccess] = useState('');

  const handleLogin = async (username, password) => {
    setLoginError('');
    try {
      const res = await api.post("/login", { username, password });
      const { access_token } = res.data;
      if (access_token) {
        localStorage.setItem('accessToken', access_token);
        localStorage.setItem('username', username);
        setIsLoggedIn(true);
        setLoggedInUsername(username);
        setLoginError('');
        return true;
      }
      setLoginError('로그인 실패: 토큰 없음');
      return false;
    } catch (e) {
      setLoginError('로그인 실패: ' + (e.response?.data?.detail || e.message));
      return false;
    }
  };

  const handleSignup = async (username, password) => {
    setSignupError('');
    setSignupSuccess('');
    try {
      await api.post("/accounts", { username, password });
      setSignupSuccess('🎉 회원가입 성공! 이제 로그인하세요.');
      return true;
    } catch (e) {
      setSignupError('회원가입 실패: ' + (e.response?.data?.detail || e.message));
      return false;
    }
  };

  const handleLogout = useCallback(() => {
    localStorage.removeItem('accessToken');
    localStorage.removeItem('username');
    setIsLoggedIn(false);
    setLoggedInUsername('');
  }, []);

  const value = {
    isLoggedIn,
    loggedInUsername,
    loginError,
    signupError,
    signupSuccess,
    handleLogin,
    handleSignup,
    handleLogout,
    setLoginError,
    setSignupError,
    setSignupSuccess,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export const useAuth = () => useContext(AuthContext);
