
import React, { createContext, useState, useContext, useCallback, useEffect } from 'react';

const UIContext = createContext();

export const UIProvider = ({ children }) => {
  const [isLoginModalOpen, setIsLoginModalOpen] = useState(false);
  const [isSignupModalOpen, setIsSignupModalOpen] = useState(false);
  const [dividerPosition, setDividerPosition] = useState(window.innerWidth * 0.4);
  const [isSidebarHidden, setIsSidebarHidden] = useState(false);
  const [showDeleteFor, setShowDeleteFor] = useState(null);
  const [isLogPanelVisible, setIsLogPanelVisible] = useState(true);

  const handleMouseMove = useCallback((e) => {
    const pos = Math.max(window.innerWidth * 0.3, Math.min(e.clientX, window.innerWidth * 0.9));
    setDividerPosition(pos);
  }, []);

  const handleMouseUp = useCallback(() => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  }, [handleMouseMove]);

  const handleMouseDown = useCallback(() => {
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [handleMouseMove, handleMouseUp]);

  useEffect(() => {
    const cleanup = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
    return cleanup;
  }, [handleMouseMove, handleMouseUp]);

  useEffect(() => {
    if (isLoginModalOpen || isSignupModalOpen) {
      const onKeyDown = e => {
        if (e.key === 'Escape') {
          setIsLoginModalOpen(false);
          setIsSignupModalOpen(false);
        }
      };
      window.addEventListener('keydown', onKeyDown);
      return () => window.removeEventListener('keydown', onKeyDown);
    }
  }, [isLoginModalOpen, isSignupModalOpen]);

  const value = {
    isLoginModalOpen, setIsLoginModalOpen,
    isSignupModalOpen, setIsSignupModalOpen,
    dividerPosition, handleMouseDown,
    isSidebarHidden, setIsSidebarHidden,
    showDeleteFor, setShowDeleteFor,
    isLogPanelVisible, setIsLogPanelVisible
  };

  return <UIContext.Provider value={value}>{children}</UIContext.Provider>;
};

export const useUI = () => useContext(UIContext);
