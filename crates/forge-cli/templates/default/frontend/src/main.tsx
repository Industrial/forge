import { createInertiaApp } from '@inertiajs/react';
import { router } from '@inertiajs/react';
import { Provider } from '@react-spectrum/s2';
import React from 'react';
import { createRoot } from 'react-dom/client';

createInertiaApp({
  resolve: (name) => {
    const pages = import.meta.glob('./pages/**/*.tsx', { eager: true });
    const path = `./pages/${name.replace(/^Pages\//, '').replace(/\./g, '/')}.tsx`;
    const mod = pages[path];
    if (!mod) throw new Error(`Page not found: ${name}`);
    const Page = (mod as { default: React.ComponentType<Record<string, unknown>> }).default;
    // Wrap each page in Spectrum Provider with Inertia router so Spectrum Link components
    // use Inertia navigation instead of full page reloads (no React Router / TanStack Router).
    function PageWithProvider(props: Record<string, unknown>) {
      return (
        <Provider
          background="base"
          router={{
            navigate: (url: string) => router.visit(url),
            useHref: (to: string) => to,
          }}
        >
          <Page {...props} />
        </Provider>
      );
    }
    return PageWithProvider;
  },
  setup({ el, App, props }) {
    createRoot(el).render(<App {...props} />);
  },
});
