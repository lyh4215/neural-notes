// src/hooks/useEditorInstance.js
import { useEditor } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';

export default function useEditorInstance() {
  return useEditor({
    extensions: [StarterKit],
    content: '<p>✍️ 여기서 글을 작성하세요</p>',
  });
}
