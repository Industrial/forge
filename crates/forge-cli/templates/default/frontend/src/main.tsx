import { createInertiaApp } from '@inertiajs/react';
import React from 'react';
import { createRoot } from 'react-dom/client';

createInertiaApp({
  resolve: (name) => {
    const pages = import.meta.glob('./pages/**/*.tsx', { eager: true });
    const path = `./pages/${name.replace(/^Pages\//, '').replace(/\./g, '/')}.tsx`;
    const mod = pages[path];
    if (!mod) throw new Error(`Page not found: ${name}`);
    return mod;
  },
  setup({ el, App, props }) {
    createRoot(el).render(<App {...props} />);
  },
});
