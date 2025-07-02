// src/hooks/useTreeBuilder.js
import { useCallback } from 'react';

export default function useTreeBuilder() {
  return useCallback((items) => {
    const root = [];
    items.forEach(post => {
      const parts = post.title.split('/');
      let nodeList = root;
      let path = '';
      parts.forEach((part, idx) => {
        path = path ? `${path}/${part}` : part;

        let node;
        if (idx === parts.length - 1) {
          // 마지막 파트는 name + postId로 구분!
          node = nodeList.find(n => n.name === part && n.postId === post.id);
          if (!node) {
            node = { name: part, children: [], postId: post.id, updatedAt: post.updated_at };
            nodeList.push(node);
          }
        } else {
          // 중간 폴더는 기존대로 name으로만 구분
          node = nodeList.find(n => n.name === part && !n.postId);
          if (!node) {
            node = { name: part, children: [] };
            nodeList.push(node);
          }
        }

        nodeList = node.children;
      });
    });
    return root;
  }, []);
}
