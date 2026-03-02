import { Content, Heading, InlineAlert } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';
import { useSession } from '../../../../context/Session';

type AuthLayoutProps = { children: React.ReactNode };

export default function AuthLayout({ children }: AuthLayoutProps) {
  const { flash } = useSession();

  return (
    <div
      className={style({
        minHeight: 'full',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 24,
      })}
    >
      <div
        className={style({
          display: 'flex',
          flexDirection: 'column',
          gap: 24,
          maxWidth: 400,
          width: 'full',
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
