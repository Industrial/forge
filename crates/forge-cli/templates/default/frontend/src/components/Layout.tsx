import { Content, Heading, InlineAlert } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';
import Navbar from './Navbar';
import { useSession } from '../context/Session';

type LayoutProps = {
  children: React.ReactNode;
  colorScheme: 'light' | 'dark';
  onToggleTheme: () => void;
};

export default function Layout({ children, colorScheme, onToggleTheme }: LayoutProps) {
  const { flash } = useSession();

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
    </div>
  );
}
