import React, { createContext, useCallback, useContext, useEffect, useState } from 'react';

export type SessionUser = { id: string; email: string };

export type Flash = { message?: string; error?: string };

export type SessionState = {
  user: SessionUser | null;
  flash: Flash | null;
  loading: boolean;
  refresh: () => Promise<void>;
};

const SessionContext = createContext<SessionState | null>(null);

export function useSession(): SessionState {
  const ctx = useContext(SessionContext);
  if (!ctx) throw new Error('useSession must be used within SessionProvider');
  return ctx;
}

async function fetchSession(): Promise<{ user: SessionUser | null; flash: Flash | null }> {
  const res = await fetch('/api/auth/session', { credentials: 'include' });
  if (!res.ok) return { user: null, flash: null };
  const data = await res.json();
  return {
    user: data.user ?? null,
    flash: data.flash && (data.flash.message != null || data.flash.error != null) ? data.flash : null,
  };
}

export function SessionProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<SessionUser | null>(null);
  const [flash, setFlash] = useState<Flash | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    const { user: u, flash: f } = await fetchSession();
    setUser(u);
    setFlash(f);
  }, []);

  useEffect(() => {
    let cancelled = false;
    fetchSession()
      .then(({ user: u, flash: f }) => {
        if (!cancelled) {
          setUser(u);
          setFlash(f);
          setLoading(false);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setUser(null);
          setFlash(null);
          setLoading(false);
        }
      });
    return () => { cancelled = true; };
  }, []);

  return (
    <SessionContext.Provider value={{ user, flash, loading, refresh }}>
      {children}
    </SessionContext.Provider>
  );
}
