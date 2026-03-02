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

import './reset.css';
import '@react-spectrum/s2/page.css';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import App from './App.tsx';

const THEME_KEY = 'spectrum-color-scheme';
if (typeof document !== 'undefined' && document.documentElement) {
  const stored = typeof localStorage !== 'undefined' ? localStorage.getItem(THEME_KEY) : null;
  const scheme = stored === 'light' || stored === 'dark' ? stored : 'dark';
  document.documentElement.setAttribute('data-color-scheme', scheme);
}

const bodyClass = style({
  display: 'flex',
  flexDirection: 'column',
  minHeight: 'full',
  height: 'full',
  overflow: 'auto',
  flexGrow: 1,
});
if (typeof document !== 'undefined' && document.body) {
  document.body.className = bodyClass;
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>,
);
