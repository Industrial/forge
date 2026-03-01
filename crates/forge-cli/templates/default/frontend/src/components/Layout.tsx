import React from 'react';
import { usePage } from '@inertiajs/react';

type SharedProps = {
  auth: { user: { id: string; email: string } | null };
  flash: { message?: string; error?: string };
  appName: string;
};

type LayoutProps = { children: React.ReactNode };

export default function Layout({ children }: LayoutProps) {
  const { auth, flash } = usePage().props as SharedProps;

  return (
    <main>
      <header>
        {auth?.user && (
          <span>Logged in as {auth.user.email}</span>
        )}
      </header>
      <article>
        {flash?.message && (
          <div className="alert alert-success" role="alert">
            {flash.message}
          </div>
        )}
        {flash?.error && (
          <div className="alert alert-error" role="alert">
            {flash.error}
          </div>
        )}
        {children}
      </article>
    </main>
  );
}
