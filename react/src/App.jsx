// src/App.jsx
import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import api from './api';
import useTreeBuilder from './hooks/useTreeBuilder';
import useEditorInstance from './hooks/useEditorInstance';

import LoginModal from './components/LoginModal';
import SignupModal from './components/SignupModal';
import NoteTree from './components/NoteTree';
import EditorPanel from './components/EditorPanel';
import RelatedNotes from './components/RelatedNotes';

const API_URL = "http://localhost:3000/posts";
const LOGIN_URL = "http://localhost:3000/login";
const SIGNUP_URL = "http://localhost:3000/accounts";

function App() {
  const [isLoginModalOpen, setIsLoginModalOpen] = useState(false);
  const [isSignupModalOpen, setIsSignupModalOpen] = useState(false);
  const [loginUsername, setLoginUsername] = useState('');
  const [loginPassword, setLoginPassword] = useState('');
  const [signupUsername, setSignupUsername] = useState('');
  const [signupPassword, setSignupPassword] = useState('');
  const [isLoggedIn, setIsLoggedIn] = useState(!!localStorage.getItem('accessToken'));
  const [loginError, setLoginError] = useState('');
  const [signupError, setSignupError] = useState('');
  const [signupSuccess, setSignupSuccess] = useState('');
  const [loggedInUsername, setLoggedInUsername] = useState(localStorage.getItem('username') || '');
  const [showDeleteFor, setShowDeleteFor] = useState(null);
  const [isLoadingPost, setIsLoadingPost] = useState(false);

  const [postId, setPostId] = useState("");
  const [userId, setUserId] = useState("");
  const [title, setTitle] = useState("");
  const [log, setLog] = useState("");
  const [posts, setPosts] = useState([]);
  const [relatedPosts, setRelatedPosts] = useState([]);
  const [searchKeyword, setSearchKeyword] = useState("");
  const [dividerPosition, setDividerPosition] = useState(window.innerWidth * 0.4);
  const [expanded, setExpanded] = useState({});
  const [isSaving, setIsSaving] = useState(false);
  const [isSidebarHidden, setIsSidebarHidden] = useState(false);

  const autoSaveTimer = useRef(null);
  const lastContentRef = useRef("");
  const lastTitleRef = useRef("");
  const isSilentUpdate = useRef(false);

  // 드래그 바
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
  useEffect(() => () => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  }, [handleMouseMove, handleMouseUp]);

  const editor = useEditorInstance(({ editor }) => {
    handleEditorOrTitleChange(title, editor.getHTML());
  });

  useEffect(() => {
    if (!editor) return;
    if (!postId) {
      editor.setEditable(false);
    } else {
      editor.setEditable(true);
    }
  }, [editor, postId]);

  const logMsg = msg => setLog(prev => `${prev}\n${msg}`);

  const buildTree = useTreeBuilder();

  const filteredPosts = useMemo(
    () => posts.filter(p => p.title.toLowerCase().includes(searchKeyword.toLowerCase())),
    [posts, searchKeyword]
  );
  const treeData = useMemo(() => buildTree(filteredPosts), [filteredPosts, buildTree]);

  const toggleExpand = (path) => {
    setExpanded(prev => ({ ...prev, [path]: !prev[path] }));
  };

  const autoSaveIfNeeded = useCallback(async (nextAction) => {
    if (autoSaveTimer.current) {
      clearTimeout(autoSaveTimer.current);
      autoSaveTimer.current = null;
    }
    if (!postId) {
      nextAction();
      return;
    }
    const curTitle = title;
    const curContent = editor?.getHTML() || "";
    if (curTitle !== lastTitleRef.current || curContent !== lastContentRef.current) {
      setIsSaving(true);
      try {
        const res = await api.put(`${API_URL}/${postId}`, {
          title: curTitle || "제목 없음",
          content: curContent,
        });
        setPosts(posts => posts.map(p => p.id === Number(postId) ? res.data : p));
        logMsg(`💾 자동저장 완료 (id: ${postId})`);
        lastTitleRef.current = curTitle;
        lastContentRef.current = curContent;
      } catch (e) {
        logMsg(`❌ 자동저장 실패: ${e.message}`);
      } finally {
        setIsSaving(false);
        nextAction();
      }
    } else {
      nextAction();
    }
  }, [postId, title, editor]);

  const loadNode = useCallback((node) => {
    if (!node.postId) return;
    autoSaveIfNeeded(async () => {
      setIsLoadingPost(true);
      try {
        const res = await api.get(`${API_URL}/${node.postId}`);
        const p = res.data;
        isSilentUpdate.current = true;
        setPostId(p.id.toString());
        setTitle(p.title);
        lastTitleRef.current = p.title;
        editor?.commands.setContent(p.content || '');
        lastContentRef.current = p.content || '';
        setRelatedPosts(p.related_posts.slice(0, 3));
        logMsg(`📄 단일 조회 완료: ${p.title}`);
      } catch (e) {
        logMsg(`❌ GET 실패: ${e.message}`);
      } finally {
        setTimeout(() => {
          isSilentUpdate.current = false;
          setIsLoadingPost(false);
        }, 100);
      }
    });
  }, [autoSaveIfNeeded, editor]);

  const handleListLoad = async () => {
    try {
      const res = await api.get(API_URL);
      setPosts(res.data);
      logMsg(`📋 리스트 로드 완료, 총 ${res.data.length}개 글`);
    } catch (e) {
      if (e.response && e.response.status === 401) {
        logMsg("세션이 만료되었습니다. 다시 로그인해주세요.");
        handleLogout();
      } else {
        logMsg(`❌ LIST 실패: ${e.message}`);
      }
    }
  };

  const restartAutoSave = (titleValue, contentValue) => {
    if (autoSaveTimer.current) {
      clearTimeout(autoSaveTimer.current);
      autoSaveTimer.current = null;
    }
    if (
      titleValue === lastTitleRef.current &&
      contentValue === lastContentRef.current
    ) {
      return;
    }
    autoSaveTimer.current = setTimeout(async () => {
      if (postId) {
        setIsSaving(true);
        try {
          const res = await api.put(`${API_URL}/${postId}`, {
            title: titleValue || "제목 없음",
            content: contentValue,
          });
          setPosts(posts => posts.map(p => p.id === Number(postId) ? res.data : p));
          logMsg(`💾 자동저장 완료 (id: ${postId})`);
          lastTitleRef.current = titleValue;
          lastContentRef.current = contentValue;
        } catch (e) {
          logMsg(`❌ 자동저장 실패: ${e.message}`);
        } finally {
          setIsSaving(false);
        }
      }
    }, 500);
  };

  const handleEditorOrTitleChange = useCallback((newTitle, newContent) => {
    if (isLoadingPost || isSilentUpdate.current) return;
    setTitle(newTitle);
    restartAutoSave(newTitle, newContent);
  }, [isLoadingPost]);

  const onTitleChange = (e) => {
    const newTitle = e.target.value;
    setTitle(newTitle);
    restartAutoSave(newTitle, editor?.getHTML() || "");
  };

  useEffect(() => {
    if (!editor) return;
    editor.on('update', ({ editor }) => {
      if (isLoadingPost || isSilentUpdate.current) return;
      restartAutoSave(title, editor.getHTML());
    });
  }, [editor, title, isLoadingPost]);

  useEffect(() => {
    return () => {
      if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
    };
  }, []);

  const handleNew = useCallback(() => {
    autoSaveIfNeeded(async () => {
      setPostId("");
      setTitle("");
      editor?.commands.setContent('<p>✍️ 여기서 글을 작성하세요</p>');
      setRelatedPosts([]);
      lastTitleRef.current = "";
      lastContentRef.current = "";

      try {
        const res = await api.post(API_URL, {
          title: "제목 없음",
          content: "",
          user_id: Number(userId),
        });
        setPosts(posts => [...posts, res.data]);
        logMsg(`✅ 새 노트 생성 완료: ID ${res.data.id}`);

        const getRes = await api.get(`${API_URL}/${res.data.id}`);
        const p = getRes.data;
        isSilentUpdate.current = true;
        setPostId(p.id.toString());
        setTitle(p.title);
        lastTitleRef.current = p.title;
        if (editor) {
          editor.commands.setContent(p.content || '');
          lastContentRef.current = p.content || '';
        }
        setRelatedPosts(p.related_posts.slice(0, 3));
        logMsg(`📄 새 노트 조회 및 편집 시작: ${p.title}`);
        setTimeout(() => {
          isSilentUpdate.current = false;
        }, 100);
      } catch (e) {
        logMsg(`❌ 새 노트 생성 실패: ${e.message}`);
      }
    });
  }, [autoSaveIfNeeded, userId, editor]);

  const handleDelete = async (delPostId) => {
    if (!delPostId) {
      logMsg('❗ 삭제할 포스트가 선택되지 않음');
      return;
    }
    try {
      await api.delete(`${API_URL}/${delPostId}`);
      logMsg(`🗑️ 삭제 완료 (id: ${delPostId})`);
      setPosts(posts => posts.filter(p => p.id !== Number(delPostId)));
      if (postId === String(delPostId)) {
        setPostId("");
        setTitle("");
        editor?.commands.setContent('<p>✍️ 여기서 글을 작성하세요</p>');
        setRelatedPosts([]);
        lastTitleRef.current = "";
        lastContentRef.current = "";
      }
    } catch (e) {
      logMsg(`❌ DELETE 실패: ${e.message}`);
    }
  };

  const handleLogin = async (e) => {
    e.preventDefault();
    setLoginError('');
    try {
      const res = await api.post(LOGIN_URL, {
        username: loginUsername,
        password: loginPassword
      });
      const { access_token } = res.data;
      if (access_token) {
        localStorage.setItem('accessToken', access_token);
        localStorage.setItem('username', loginUsername);
        setIsLoggedIn(true);
        setLoggedInUsername(loginUsername);
        setIsLoginModalOpen(false);
        setLoginUsername('');
        setLoginPassword('');
        setLoginError('');
        logMsg(`✅ 로그인 성공: ${loginUsername}`);
        handleListLoad();
      } else {
        setLoginError('로그인 실패: 토큰 없음');
      }
    } catch (e) {
      setLoginError('로그인 실패: ' + (e.response?.data?.detail || e.message));
    }
  };

  const handleSignup = async (e) => {
    e.preventDefault();
    setSignupError('');
    setSignupSuccess('');
    try {
      await api.post(SIGNUP_URL, {
        username: signupUsername,
        password: signupPassword
      });
      setSignupSuccess('🎉 회원가입 성공! 이제 로그인하세요.');
      setSignupUsername('');
      setSignupPassword('');
    } catch (e) {
      setSignupError('회원가입 실패: ' + (e.response?.data?.detail || e.message));
    }
  };

  const handleLogout = () => {
    localStorage.removeItem('accessToken');
    localStorage.removeItem('username');
    setIsLoggedIn(false);
    setLoggedInUsername('');
    setUserId('');
    setPostId('');
    setTitle('');
    setLog('');
    setPosts([]);
    setRelatedPosts([]);
    setSearchKeyword('');
    editor?.commands.setContent('<p>✍️ 여기서 글을 작성하세요</p>');
    logMsg('👋 로그아웃됨');
  };

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

  useEffect(() => {
    if (isLoggedIn) {
      handleListLoad();
    }
  }, [isLoggedIn]);

  // ---- return ----
  return (
    <div style={{
      display: 'flex', padding: 20, fontFamily: 'Arial', height: '100vh',
      width: '100vw', boxSizing: 'border-box', background: '#1e1e1e', userSelect: 'none',
      position: 'relative'
    }}>
      {/* 사이드바 */}
      {!isSidebarHidden &&
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
              expanded={expanded}
              toggleExpand={toggleExpand}
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
          {/* 좌하단 숨기기 버튼 */}
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
      }

      {/* 드래그바 */}
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

      {/* 에디터/관련노트/좌하단 나타내기 버튼 */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', position: 'relative' }}>
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
        {/* 좌하단 나타내기 버튼 */}
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
            ➡ 사이드바 나타내기
          </button>
        }
      </div>

      {/* 모달들 */}
      <LoginModal
        isOpen={isLoginModalOpen}
        onClose={() => setIsLoginModalOpen(false)}
        onSubmit={handleLogin}
        username={loginUsername}
        setUsername={setLoginUsername}
        password={loginPassword}
        setPassword={setLoginPassword}
        error={loginError}
      />
      <SignupModal
        isOpen={isSignupModalOpen}
        onClose={() => setIsSignupModalOpen(false)}
        onSubmit={handleSignup}
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

export default App;
