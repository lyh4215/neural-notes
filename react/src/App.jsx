import React from 'react';
import useEditorInstance from './hooks/useEditorInstance';
import Layout from './components/Layout';
import { NotesProvider } from './contexts/NotesContext';

function App() {
  const editor = useEditorInstance();

  return (
    <NotesProvider editor={editor}>
      <Layout editor={editor} />
    </NotesProvider>
  );
}

export default App;