import { Content, Heading, InlineAlert } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';
import { useLocation } from 'react-router-dom';
import AppShell from './AppShell';
import Navbar from './Navbar';
import Sidebar from './Sidebar';
import { useSession } from '../context/Session';

type LayoutProps = { children: React.ReactNode };

function pathnameToSidebarActiveKey(pathname: string): string | undefined {
  if (pathname === '/' || pathname === '') return 'home';
  if (pathname.startsWith('/photos')) return 'photos';
  if (pathname.startsWith('/ideas')) return 'ideas';
  return undefined;
}

export default function Layout({ children }: LayoutProps) {
  const { user, flash } = useSession();
  const location = useLocation();
  const pathname = location.pathname;
  const sidebarActiveKey = pathnameToSidebarActiveKey(pathname);

  return (
    <AppShell
      navbar={
        <Navbar appName="App" user={user ?? undefined} />
      }
      sidebar={<Sidebar activeKey={sidebarActiveKey} />}
    >
      <div
        className={style({
          display: 'flex',
          flexDirection: 'column',
          gap: 16,
          margin: 16,
          backgroundColor: 'layer-1',
          padding: 16,
          borderRadius: 'default',
        })}
      >
        {(flash?.message ?? flash?.error) != null && (
          <>
            {flash?.message != null && (
              <InlineAlert variant="positive">
                <Content>{flash.message}</Content>
              </InlineAlert>
            )}
            {flash?.error != null && (
              <InlineAlert variant="negative">
                <Heading>Error</Heading>
                <Content>{flash.error}</Content>
              </InlineAlert>
            )}
          </>
        )}
        {children}
      </div>
    </AppShell>
  );
}
