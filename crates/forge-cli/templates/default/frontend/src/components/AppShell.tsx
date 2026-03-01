import React from 'react';

type AppShellProps = {
  navbar: React.ReactNode;
  sidebar: React.ReactNode;
  children: React.ReactNode;
};

/**
 * Three-region layout: fixed top navbar, fixed left sidebar, scrollable main content.
 * Uses full viewport height. Spectrum 2: div + inline style (macro doesn't support 100vh/flex shorthand).
 */
export default function AppShell({ navbar, sidebar, children }: AppShellProps) {
  return (
    <div
      style={{
        height: '100vh',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
    >
      <header style={{ flex: '0 0 auto' }}>{navbar}</header>
      <div
        style={{
          display: 'flex',
          flexDirection: 'row',
          flex: 1,
          minHeight: 0,
          overflow: 'hidden',
        }}
      >
        <aside style={{ flex: '0 0 auto', overflow: 'auto' }}>{sidebar}</aside>
        <main style={{ flex: 1, overflow: 'auto', minWidth: 0 }}>{children}</main>
      </div>
    </div>
  );
}
