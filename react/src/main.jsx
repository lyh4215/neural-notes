import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';
import 'aframe';
import App from './App.jsx';
import { AuthProvider } from './contexts/AuthContext';
import { UIProvider } from './contexts/UIContext';

import { BrowserRouter } from 'react-router-dom';

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <BrowserRouter>
      <AuthProvider>
        <UIProvider>
          <App />
        </UIProvider>
      </AuthProvider>
    </BrowserRouter>
  </StrictMode>,
);