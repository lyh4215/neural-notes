// src/hooks/useTreeBuilder.js
import { useCallback } from 'react';

export default function useTreeBuilder() {
  return useCallback((items) => {
    const root = [];
    items.forEach(post => {
      const parts = post.title.split('/');
      let currentLevel = root;
      let currentPath = '';

      parts.forEach((part, idx) => {
        currentPath = currentPath ? `${currentPath}/${part}` : part;

        let node = currentLevel.find(n => n.name === part && (idx === parts.length - 1 ? n.postId === post.id : n.postId === undefined));

        if (!node) {
          if (idx === parts.length - 1) {
            node = { name: part, children: [], postId: post.id, updatedAt: post.updated_at };
          } else {
            node = { name: part, children: [] };
          }
          currentLevel.push(node);
        }
        currentLevel = node.children;
      });
    });
    return root;
  }, []);
}
