/*
 * Copyright 2024 Adobe. All rights reserved.
 * This file is licensed to you under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License. You may obtain a copy
 * of the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software distributed under
 * the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR REPRESENTATIONS
 * OF ANY KIND, either express or implied. See the License for the specific language
 * governing permissions and limitations under the License.
 */

import { useEffect, useState } from 'react';
import { Route, Routes, useNavigate } from 'react-router-dom';
import { Provider } from '@react-spectrum/s2';
import type { NavigateOptions } from 'react-router-dom';
import { SessionProvider } from './context/Session';
import Layout from './components/Layout';
import AuthLayout from './components/AuthLayout';
import ProtectedRoute from './components/ProtectedRoute';
import Login from './pages/Auth/Login';
import Register from './pages/Auth/Register';
import Index from './pages/Index';
import Dashboard from './pages/Dashboard';
import Profile from './pages/Profile';

declare module '@react-spectrum/s2' {
  interface RouterConfig {
    routerOptions: NavigateOptions;
  }
}

const STORAGE_KEY = 'spectrum-color-scheme';

function App() {
  const navigate = useNavigate();
  const [colorScheme, setColorScheme] = useState<'light' | 'dark'>(() => {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored === 'light' || stored === 'dark') return stored;
    }
    return 'dark';
  });

  useEffect(() => {
    document.documentElement.setAttribute('data-color-scheme', colorScheme);
    if (typeof window !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, colorScheme);
    }
  }, [colorScheme]);

  const layoutProps = {
    colorScheme,
    onToggleTheme: () => setColorScheme((s) => (s === 'dark' ? 'light' : 'dark')),
  };

  return (
    <SessionProvider>
      <Provider
        elementType="main"
        locale="en-US"
        colorScheme={colorScheme}
        router={{
          navigate: (url: string) => navigate(url),
          useHref: (to: string) => to,
        }}
      >
        <Routes>
          <Route path="/login" element={<AuthLayout><Login /></AuthLayout>} />
          <Route path="/register" element={<AuthLayout><Register /></AuthLayout>} />
          <Route
            path="/"
            element={
              <ProtectedRoute>
                <Layout {...layoutProps}><Index /></Layout>
              </ProtectedRoute>
            }
          />
          <Route
            path="/dashboard"
            element={
              <ProtectedRoute>
                <Layout {...layoutProps}><Dashboard /></Layout>
              </ProtectedRoute>
            }
          />
          <Route
            path="/profile"
            element={
              <ProtectedRoute>
                <Layout {...layoutProps}><Profile /></Layout>
              </ProtectedRoute>
            }
          />
        </Routes>
      </Provider>
    </SessionProvider>
  );
}

export default App;
