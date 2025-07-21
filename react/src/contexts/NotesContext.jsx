

import React, { createContext, useState, useContext, useCallback, useMemo, useRef, useEffect } from 'react';
import api from '../api';
import useTreeBuilder from '../hooks/useTreeBuilder';
import { useNavigate } from 'react-router-dom';
import { useAuth } from './AuthContext';
import { useTranslation } from 'react-i18next';

const NotesContext = createContext();


export const NotesProvider = ({ children, editor }) => {
  const { t } = useTranslation();
  const { isLoggedIn, handleLogout } = useAuth();
  const navigate = useNavigate();
  const [posts, setPosts] = useState([]);
  const [postId, setPostId] = useState("");
  const [title, setTitle] = useState("");
  const [relatedPosts, setRelatedPosts] = useState([]);
  const [searchKeyword, setSearchKeyword] = useState("");
  
  const [log, setLog] = useState([]);
  const logMsg = useCallback((msg) => setLog(prev => [...prev, `${new Date().toLocaleTimeString()} - ${msg}`]), []);

  const [isSaving, setIsSaving] = useState(false);
  const [isLoadingPost, setIsLoadingPost] = useState(false);
  const [isLoadingList, setIsLoadingList] = useState(false);
  const [listError, setListError] = useState(null);

  const autoSaveTimer = useRef(null);
  const lastContentRef = useRef("");
  const lastTitleRef = useRef("");
  const isSilentUpdate = useRef(false);

  
  const buildTree = useTreeBuilder();

  const filteredPosts = useMemo(
    () => (Array.isArray(posts) ? posts.filter(p => p.title.toLowerCase().includes(searchKeyword.toLowerCase())) : []),
    [posts, searchKeyword]
  );
  const treeData = useMemo(() => buildTree(filteredPosts), [filteredPosts, buildTree]);

  const handleListLoad = useCallback(async () => {
    if (!isLoggedIn) {
      setPosts([]); // {t('clear_list_on_logout')}
      setListError(null); // 에러 상태 초기화
      return;
    }
    setIsLoadingList(true);
    setListError(null); // 새로운 로드 시작 시 에러 상태 초기화
    try {
      const res = await api.get("/posts");
      if (Array.isArray(res.data)) {
        
        setPosts(res.data);
        logMsg(t('list_load_complete', { count: res.data.length }));
      } else {
        console.error(t('api_response_not_array'), res.data);
        setPosts([]); // 배열이 아니면 비우기
        logMsg(t('list_fail_invalid_data_format'));
      }
    } catch (e) {
      if (e.response && e.response.status === 401) {
        logMsg(t('session_expired'));
        handleLogout();
      } else {
        const errorMessage = e.code === 'ERR_NETWORK' ? t('backend_connection_failed') : (e.response?.data?.detail || e.message);
        logMsg(t('list_fail', { message: errorMessage }));
        setListError(t('failed_to_load_notes') + ` (${errorMessage})`);
      }
      setPosts([]); // 에러 발생 시 비우기
    } finally {
      setIsLoadingList(false);
    }
  }, [isLoggedIn, handleLogout, logMsg]);

  useEffect(() => {
    handleListLoad();
  }, [isLoggedIn, handleListLoad]);

  const autoSaveIfNeeded = useCallback(async (nextAction) => {
    if (autoSaveTimer.current) {
      clearTimeout(autoSaveTimer.current);
      autoSaveTimer.current = null;
    }
    if (!postId || !editor) {
      nextAction();
      return;
    }
    const curTitle = title;
    const curContent = editor.getHTML() || "";
    if (curTitle !== lastTitleRef.current || curContent !== lastContentRef.current) {
      setIsSaving(true);
      try {
        const res = await api.put(`/posts/${postId}`, { title: curTitle || t('untitled'), content: curContent });
        setPosts(posts => Array.isArray(posts) ? posts.map(p => p.id === Number(postId) ? res.data : p) : [res.data]);
        logMsg(t('autosave_complete', { postId }));
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
    
    if (!node.postId || !editor) {
      
      return;
    }
    autoSaveIfNeeded(async () => {
      setIsLoadingPost(true);
      try {
        
        const res = await api.get(`/posts/${node.postId}`);
        const p = res.data;
        isSilentUpdate.current = true;
        setPostId(p.id.toString());
        setTitle(p.title);
        lastTitleRef.current = p.title;
        
        editor.commands.setContent(p.content || '');
        lastContentRef.current = p.content || '';
        setRelatedPosts(p.related_posts.slice(0, 3));
        logMsg(t('single_view_complete', { title: p.title }));
        navigate(`/posts/${p.id}`);
      } catch (e) {
        logMsg(t('get_failed', { message: e.message }));
        console.error('GET failed:', e);
      } finally {
        setIsLoadingPost(false);
      }
    });
  }, [autoSaveIfNeeded, editor, navigate]);

  const handleNew = useCallback(() => {
    if (!editor) return;
  
    autoSaveIfNeeded(async () => {
      isSilentUpdate.current = true; // 자동 저장 비활성화
  
      try {
        // 1. 새 노트 생성 요청
        const res = await api.post("/posts", { title: t('untitled'), content: "" });
        const newPost = res.data;
        logMsg(t('new_note_created', { id: newPost.id }));
  
        // 2. 전체 포스트 목록에 새 노트 추가
        setPosts(prevPosts => [...prevPosts, newPost]);
  
        // 3. UI 상태를 새 노트 기준으로 즉시 업데이트
        setPostId(newPost.id.toString());
        setTitle(newPost.title);
        editor.commands.setContent(newPost.content || t('write_here'));
        setRelatedPosts([]);
  
        // 4. 마지막 저장 상태를 새 노트 기준으로 업데이트
        lastTitleRef.current = newPost.title;
        lastContentRef.current = newPost.content || '';
  
        // 5. URL 변경
        navigate(`/posts/${newPost.id}`);
  
      } catch (e) {
        logMsg(t('new_note_creation_failed', { message: e.message }));
      } finally {
        // 6. 짧은 지연 후 자동 저장 다시 활성화

      }
    });
  }, [autoSaveIfNeeded, editor, navigate]);

  const handleDelete = async (delPostId) => {
    if (!delPostId) return;
    try {
      await api.delete(`/posts/${delPostId}`);
      logMsg(t('delete_complete', { id: delPostId }));
      setPosts(p => p.filter(p => p.id !== Number(delPostId)));
      if (postId === String(delPostId)) {
        setPostId("");
        setTitle("");
        editor?.commands.setContent(t('write_here'));
        setRelatedPosts([]);
        lastTitleRef.current = "";
        lastContentRef.current = "";
        navigate(`/`);
      }
    } catch (e) {
      logMsg(t('delete_failed', { message: e.message }));
    }
  };

  const restartAutoSave = useCallback((titleValue, contentValue) => {
    if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
    if (titleValue === lastTitleRef.current && contentValue === lastContentRef.current) return;

    autoSaveTimer.current = setTimeout(async () => {
      if (postId && editor) {
        setIsSaving(true);
        try {
          const res = await api.put(`/posts/${postId}`, { title: titleValue || t('untitled'), content: contentValue });
          setPosts(posts => Array.isArray(posts) ? posts.map(p => p.id === Number(postId) ? res.data : p) : [res.data]);
          logMsg(t('autosave_complete', { postId }));
          lastTitleRef.current = titleValue;
          lastContentRef.current = contentValue;
        } catch (e) {
          logMsg(t('autosave_failed', { message: e.message }));
        } finally {
          setIsSaving(false);
        }
      }
    }, 500);
  }, [postId, editor]);

  useEffect(() => {
    if (!editor) return;
    const handleUpdate = ({ editor }) => {
      if (isLoadingPost || isSilentUpdate.current) return;
      restartAutoSave(title, editor.getHTML());
    };
    editor.on('update', handleUpdate);
    return () => editor.off('update', handleUpdate);
  }, [editor, title, isLoadingPost, restartAutoSave]);

  useEffect(() => {
    if (isSilentUpdate.current) {
      const timer = setTimeout(() => {
        isSilentUpdate.current = false;
      }, 0); // 다음 렌더링 사이클에 실행
      return () => clearTimeout(timer);
    }
  }, [postId]);

  const onTitleChange = (e) => {
    const newTitle = e.target.value;
    setTitle(newTitle);
    restartAutoSave(newTitle, editor?.getHTML() || "");
  };

  const value = {
    posts, setPosts, postId, setPostId, title, setTitle, onTitleChange,
    relatedPosts, setRelatedPosts, searchKeyword, setSearchKeyword,
    handleListLoad, loadNode, handleNew, handleDelete, isSilentUpdate, log, isSaving, isLoadingList, listError, treeData
  };

  return <NotesContext.Provider value={value}>{children}</NotesContext.Provider>;
};

export const useNotes = () => useContext(NotesContext);
