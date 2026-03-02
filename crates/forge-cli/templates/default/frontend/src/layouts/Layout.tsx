import { Content, Heading, InlineAlert } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';
import { useLocation } from 'react-router-dom';
import Navbar from '../components/Navbar';
import { useSession } from '../context/Session';

type LayoutProps = {
  children: React.ReactNode;
  colorScheme: 'light' | 'dark';
  onToggleTheme: () => void;
};

function FlashMessages() {
  const { flash } = useSession();
  if ((flash?.message ?? flash?.error) == null) return null;
  return (
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
  );
}

export default function Layout({ children, colorScheme, onToggleTheme }: LayoutProps) {
  const { pathname } = useLocation();
  const isDashboard = pathname.startsWith('/dashboard');

  return (
    <div
      className={style({
        display: 'flex',
        flexDirection: 'column',
        minHeight: 'full',
        height: 'full',
        overflow: 'auto',
        flexGrow: 1,
      })}
    >
      <Navbar
        appName="App"
        colorScheme={colorScheme}
        onToggleTheme={onToggleTheme}
      />
      {isDashboard ? (
        <div
          className={style({
            display: 'flex',
            flexDirection: 'column',
            flexGrow: 1,
            minHeight: 0,
          })}
        >
          <FlashMessages />
          {children}
        </div>
      ) : (
        <div
          className={style({
            display: 'flex',
            flexDirection: 'column',
            gap: 16,
            margin: 16,
            backgroundColor: 'layer-1',
            padding: 16,
            borderRadius: 'default',
            flexGrow: 1,
          })}
        >
          <FlashMessages />
          {children}
        </div>
      )}
    </div>
  );
}
