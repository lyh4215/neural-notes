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
  const [isSearchMode, setIsSearchMode] = useState(false);

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
    () => {
      if (isSearchMode) {
        return posts; // When in search mode, display all results from the backend
      } else {
        // In regular mode, filter by title based on searchKeyword
        return Array.isArray(posts) ? posts.filter(p => p.title.toLowerCase().includes(searchKeyword.toLowerCase())) : [];
      }
    },
    [posts, searchKeyword, isSearchMode]
  );
  const treeData = useMemo(() => buildTree(filteredPosts), [filteredPosts, buildTree]);

  const handleListLoad = useCallback(async () => {
    if (!isLoggedIn) {
      setPosts([]);
      setListError(null);
      return;
    }
    setIsLoadingList(true);
    setListError(null);
    try {
      const res = await api.get("/posts");
      if (Array.isArray(res.data)) {
        setPosts(res.data);
        logMsg(t('list_load_complete', { count: res.data.length }));
      } else {
        console.error(t('api_response_not_array'), res.data);
        setPosts([]);
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
      setPosts([]);
    } finally {
      setIsLoadingList(false);
      setIsSearchMode(false);
    }
  }, [isLoggedIn, handleLogout, logMsg, t]);

  useEffect(() => {
    handleListLoad();
  }, [isLoggedIn, handleListLoad]);

  const handleSearch = async () => {
    if (!searchKeyword.trim()) {
      handleListLoad();
      return;
    }
    setIsLoadingList(true);
    setListError(null);
    try {
      const res = await api.get(`/posts/search?q=${encodeURIComponent(searchKeyword)}`);
      setPosts(res.data);
      setIsSearchMode(true);
      logMsg(`Search completed for: "${searchKeyword}"`);
    } catch (e) {
      const errorMessage = e.response?.data?.detail || e.message;
      logMsg(`Search failed: ${errorMessage}`);
      setListError(`Search failed: ${errorMessage}`);
      setPosts([]);
    } finally {
      setIsLoadingList(false);
    }
  };

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
  }, [postId, title, editor, t, logMsg]);

  const loadNode = useCallback((node) => {
    if (!node.postId || !editor) return;
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
  }, [autoSaveIfNeeded, editor, navigate, logMsg, t]);

  const handleNew = useCallback(() => {
    if (!editor) return;
    autoSaveIfNeeded(async () => {
      isSilentUpdate.current = true;
      try {
        const res = await api.post("/posts", { title: t('untitled'), content: "" });
        const newPost = res.data;
        logMsg(t('new_note_created', { id: newPost.id }));
        setPosts(prevPosts => [...prevPosts, newPost]);
        setPostId(newPost.id.toString());
        setTitle(newPost.title);
        editor.commands.setContent(newPost.content || t('write_here'));
        setRelatedPosts([]);
        lastTitleRef.current = newPost.title;
        lastContentRef.current = newPost.content || '';
        navigate(`/posts/${newPost.id}`);
      } catch (e) {
        logMsg(t('new_note_creation_failed', { message: e.message }));
      }
    });
  }, [autoSaveIfNeeded, editor, navigate, logMsg, t]);

  const handleDelete = useCallback(async (delPostId) => {
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
  }, [postId, editor, navigate, logMsg, t]);

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
  }, [postId, editor, logMsg, t]);

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
      }, 0);
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
    handleListLoad, loadNode, handleNew, handleDelete, isSilentUpdate, log, isSaving, isLoadingList, listError, treeData,
    handleSearch, isSearchMode
  };

  return <NotesContext.Provider value={value}>{children}</NotesContext.Provider>;
};

export const useNotes = () => useContext(NotesContext);