

import React, { createContext, useState, useContext, useCallback, useMemo, useRef, useEffect } from 'react';
import api from '../api';
import useTreeBuilder from '../hooks/useTreeBuilder';
import { useAuth } from './AuthContext';

const NotesContext = createContext();
const API_URL = "http://localhost:3000/posts";

export const NotesProvider = ({ children, editor }) => {
  const { isLoggedIn, handleLogout } = useAuth();
  const [posts, setPosts] = useState([]);
  const [postId, setPostId] = useState("");
  const [title, setTitle] = useState("");
  const [relatedPosts, setRelatedPosts] = useState([]);
  const [searchKeyword, setSearchKeyword] = useState("");
  const [log, setLog] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [isLoadingPost, setIsLoadingPost] = useState(false);

  const autoSaveTimer = useRef(null);
  const lastContentRef = useRef("");
  const lastTitleRef = useRef("");
  const isSilentUpdate = useRef(false);

  const logMsg = msg => setLog(prev => `${prev}\n${msg}`);
  const buildTree = useTreeBuilder();

  const filteredPosts = useMemo(
    () => posts.filter(p => p.title.toLowerCase().includes(searchKeyword.toLowerCase())),
    [posts, searchKeyword]
  );
  const treeData = useMemo(() => buildTree(filteredPosts), [filteredPosts, buildTree]);

  const handleListLoad = useCallback(async () => {
    if (!isLoggedIn) return;
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
  }, [isLoggedIn, handleLogout]);

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
        const res = await api.put(`${API_URL}/${postId}`, { title: curTitle || "제목 없음", content: curContent });
        setPosts(p => p.map(p => p.id === Number(postId) ? res.data : p));
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
    if (!node.postId || !editor) return;
    autoSaveIfNeeded(async () => {
      setIsLoadingPost(true);
      try {
        const res = await api.get(`${API_URL}/${node.postId}`);
        const p = res.data;
        isSilentUpdate.current = true;
        setPostId(p.id.toString());
        setTitle(p.title);
        lastTitleRef.current = p.title;
        editor.commands.setContent(p.content || '');
        lastContentRef.current = p.content || '';
        setRelatedPosts(p.related_posts.slice(0, 3));
        logMsg(`📄 단일 조회 완료: ${p.title}`);
      } catch (e) {
        logMsg(`❌ GET 실패: ${e.message}`);
      } finally {
        setTimeout(() => { isSilentUpdate.current = false; setIsLoadingPost(false); }, 100);
      }
    });
  }, [autoSaveIfNeeded, editor]);

  const handleNew = useCallback(() => {
    if (!editor) return;
    autoSaveIfNeeded(async () => {
      setPostId("");
      setTitle("");
      editor.commands.setContent('<p>✍️ 여기서 글을 작성하세요</p>');
      setRelatedPosts([]);
      lastTitleRef.current = "";
      lastContentRef.current = "";

      try {
        const res = await api.post(API_URL, { title: "제목 없음", content: "" });
        setPosts(p => [...p, res.data]);
        logMsg(`✅ 새 노트 생성 완료: ID ${res.data.id}`);
        loadNode({ postId: res.data.id });
      } catch (e) {
        logMsg(`❌ 새 노트 생성 실패: ${e.message}`);
      }
    });
  }, [autoSaveIfNeeded, editor, loadNode]);

  const handleDelete = async (delPostId) => {
    if (!delPostId) return;
    try {
      await api.delete(`${API_URL}/${delPostId}`);
      logMsg(`🗑️ 삭제 완료 (id: ${delPostId})`);
      setPosts(p => p.filter(p => p.id !== Number(delPostId)));
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

  const restartAutoSave = useCallback((titleValue, contentValue) => {
    if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
    if (titleValue === lastTitleRef.current && contentValue === lastContentRef.current) return;

    autoSaveTimer.current = setTimeout(async () => {
      if (postId && editor) {
        setIsSaving(true);
        try {
          const res = await api.put(`${API_URL}/${postId}`, { title: titleValue || "제목 없음", content: contentValue });
          setPosts(p => p.map(p => p.id === Number(postId) ? res.data : p));
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

  const onTitleChange = (e) => {
    const newTitle = e.target.value;
    setTitle(newTitle);
    restartAutoSave(newTitle, editor?.getHTML() || "");
  };

  const value = {
    posts, setPosts, postId, setPostId, title, setTitle, onTitleChange,
    relatedPosts, setRelatedPosts, searchKeyword, setSearchKeyword,
    log, setLog, logMsg, isSaving, isLoadingPost, treeData,
    handleListLoad, loadNode, handleNew, handleDelete, isSilentUpdate
  };

  return <NotesContext.Provider value={value}>{children}</NotesContext.Provider>;
};

export const useNotes = () => useContext(NotesContext);
