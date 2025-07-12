import React from 'react';
import { Routes, Route } from 'react-router-dom';
import useEditorInstance from './hooks/useEditorInstance';
import Layout from './components/Layout';
import { NotesProvider } from './contexts/NotesContext';

function App() {
  const editor = useEditorInstance();

  return (
    <NotesProvider editor={editor}>
      <Routes>
        <Route path="/" element={<Layout editor={editor} />} />
        <Route path="/posts/:id" element={<Layout editor={editor} />} />
      </Routes>
    </NotesProvider>
  );
}

export default App;