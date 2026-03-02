import { Content, Heading, InlineAlert } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import { usePage } from '@inertiajs/react';
import React from 'react';

type SharedProps = {
  flash?: { message?: string; error?: string };
};

type AuthLayoutProps = { children: React.ReactNode };

/**
 * Centered layout for auth pages (login, register). No navbar or sidebar.
 * Content is centered horizontally and vertically in the viewport.
 */
export default function AuthLayout({ children }: AuthLayoutProps) {
  const page = usePage();
  const flash = (page.props as SharedProps).flash;

  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 24,
      }}
    >
      <div
        className={style({
          display: 'flex',
          flexDirection: 'column',
          gap: 24,
          maxWidth: 400,
        })}
        style={{ width: '100%' }}
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
    </div>
  );
}
