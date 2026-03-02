import { Content, Heading, InlineAlert } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import { usePage } from '@inertiajs/react';
import React from 'react';
import AppShell from './AppShell';
import Navbar from './Navbar';
import Sidebar from './Sidebar';

type SharedProps = {
  auth: { user: { id: string; email: string } | null };
  flash: { message?: string; error?: string };
  appName: string;
};

type LayoutProps = { children: React.ReactNode };

function pathnameToSidebarActiveKey(pathname: string): string | undefined {
  if (pathname === '/' || pathname === '') return 'home';
  if (pathname.startsWith('/photos')) return 'photos';
  if (pathname.startsWith('/ideas')) return 'ideas';
  return undefined;
}

export default function Layout({ children }: LayoutProps) {
  const page = usePage();
  const { auth, flash, appName } = page.props as SharedProps;
  const pathname =
    typeof page.url === 'string'
      ? (() => {
          try {
            return new URL(page.url, 'http://_').pathname;
          } catch {
            return '/';
          }
        })()
      : '/';
  const sidebarActiveKey = pathnameToSidebarActiveKey(pathname);

  return (
    <AppShell
      navbar={
        <Navbar appName={appName} user={auth?.user ?? undefined} />
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
            {flash.message != null && (
              <InlineAlert variant="positive">
                <Content>{flash.message}</Content>
              </InlineAlert>
            )}
            {flash.error != null && (
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
