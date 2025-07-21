import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';
import 'aframe';
import App from './App.jsx';
import { AuthProvider } from './contexts/AuthContext';
import { UIProvider } from './contexts/UIContext';
import { BrowserRouter } from 'react-router-dom';
import { I18nextProvider } from 'react-i18next';
import i18n from './i18n'; // i18n 설정 파일 임포트

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <BrowserRouter>
      <AuthProvider>
        <UIProvider>
          <I18nextProvider i18n={i18n}>
            <App />
          </I18nextProvider>
        </UIProvider>
      </AuthProvider>
    </BrowserRouter>
  </StrictMode>,
);