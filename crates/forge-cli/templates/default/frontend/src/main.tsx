import '@react-spectrum/s2/page.css';
import { Provider } from '@react-spectrum/s2';
import React from 'react';

// Set theme on <html> before render so Spectrum design tokens apply (page.css + Provider need this).
if (typeof document !== 'undefined' && document.documentElement) {
  document.documentElement.setAttribute('data-color-scheme', 'dark');
  document.documentElement.setAttribute('data-background', 'base');
}
import { createRoot } from 'react-dom/client';
import { BrowserRouter, Route, Routes, useNavigate } from 'react-router-dom';
import { SessionProvider } from './context/Session';
import Layout from './components/Layout';
import AuthLayout from './components/AuthLayout';
import Home from './pages/Home';
import Login from './pages/Auth/Login';
import Register from './pages/Auth/Register';
import Dashboard from './pages/Dashboard';
import WsDemo from './pages/WsDemo';
import Photos from './pages/Photos';
import Ideas from './pages/Ideas';
import Files from './pages/Files';

function Providers({ children }: { children: React.ReactNode }) {
  const navigate = useNavigate();

  return (
    <SessionProvider>
      <Provider
        elementType="main"
        background="base"
        colorScheme="dark"
        router={{
          navigate: (url: string) => navigate(url),
          useHref: (to: string) => to,
        }}
      >
        {children}
      </Provider>
    </SessionProvider>
  );
}

function App() {
  return (
    <Providers>
      <Routes>
        <Route path="/" element={<Layout><Home /></Layout>} />
        <Route path="/login" element={<AuthLayout><Login /></AuthLayout>} />
        <Route path="/register" element={<AuthLayout><Register /></AuthLayout>} />
        <Route path="/dashboard" element={<Layout><Dashboard /></Layout>} />
        <Route path="/ws-demo" element={<Layout><WsDemo /></Layout>} />
        <Route path="/photos" element={<Layout><Photos /></Layout>} />
        <Route path="/ideas" element={<Layout><Ideas /></Layout>} />
        <Route path="/files" element={<Layout><Files /></Layout>} />
      </Routes>
    </Providers>
  );
}

const el = document.getElementById('root');
if (el) {
  createRoot(el).render(
    <React.StrictMode>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </React.StrictMode>
  );
}
