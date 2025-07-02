// src/components/LoginModal.jsx
import React from 'react';

export default function LoginModal({
  isOpen, onClose, onSubmit, username, setUsername, password, setPassword, error
}) {
  if (!isOpen) return null;
  return (
    <div style={{
      position: 'fixed', zIndex: 2000, left: 0, top: 0, width: '100vw', height: '100vh',
      background: 'rgba(0,0,0,0.7)', display: 'flex', alignItems: 'center', justifyContent: 'center'
    }}>
      <form onSubmit={onSubmit} style={{
        background: '#222', borderRadius: 12, padding: 32, minWidth: 320, boxShadow: '0 4px 24px #0009',
        display: 'flex', flexDirection: 'column', alignItems: 'stretch', gap: 18, border: '1px solid #444'
      }}>
        <h2 style={{ color: '#fff', margin: 0, textAlign: 'center' }}>로그인</h2>
        <input type="text" required autoFocus placeholder="Username" value={username} onChange={e => setUsername(e.target.value)}
          style={{ fontSize: 16, padding: 8, borderRadius: 6, border: '1px solid #333', background: '#2e2e2e', color: '#fff' }} />
        <input type="password" required placeholder="Password" value={password} onChange={e => setPassword(e.target.value)}
          style={{ fontSize: 16, padding: 8, borderRadius: 6, border: '1px solid #333', background: '#2e2e2e', color: '#fff' }} />
        {error && <div style={{ color: '#f44', fontWeight: 600 }}>{error}</div>}
        <div style={{ display: 'flex', justifyContent: 'center', gap: 12 }}>
          <button type="submit" style={{ padding: '8px 24px', borderRadius: 6, fontSize: 16 }}>로그인</button>
          <button type="button" onClick={onClose} style={{ padding: '8px 24px', borderRadius: 6, fontSize: 16, background: '#444', color: '#fff' }}>취소</button>
        </div>
      </form>
    </div>
  );
}
