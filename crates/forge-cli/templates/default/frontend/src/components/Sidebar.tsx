import { Button, Divider, Link, Text } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';

export type SidebarNavItem = {
  key: string;
  href: string;
  label: string;
  icon: React.ReactNode;
};

const DEFAULT_NAV_ITEMS: SidebarNavItem[] = [
  { key: 'home', href: '/', label: 'Home', icon: <HomeIcon /> },
  { key: 'photos', href: '/photos', label: 'Photos', icon: <PhotosIcon /> },
  { key: 'ideas', href: '/ideas', label: 'Ideas', icon: <IdeasIcon /> },
];

function HomeIcon() {
  return (
    <span aria-hidden style={{ width: 20, height: 20, display: 'block' }}>
      <svg viewBox="0 0 24 24" fill="currentColor" width="100%" height="100%">
        <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z" />
      </svg>
    </span>
  );
}

function PhotosIcon() {
  return (
    <span aria-hidden style={{ width: 20, height: 20, display: 'block' }}>
      <svg viewBox="0 0 24 24" fill="currentColor" width="100%" height="100%">
        <path d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z" />
      </svg>
    </span>
  );
}

function IdeasIcon() {
  return (
    <span aria-hidden style={{ width: 20, height: 20, display: 'block' }}>
      <svg viewBox="0 0 24 24" fill="currentColor" width="100%" height="100%">
        <path d="M9 21c0 .5.4 1 1 1h4c.6 0 1-.5 1-1v-1H9v1zm3-19C8.1 2 5 5.1 5 9c0 2.4 1.2 4.5 3 5.7V17c0 .5.4 1 1 1h6c.6 0 1-.5 1-1v-2.3c1.8-1.3 3-3.4 3-5.7 0-3.9-3.1-7-7-7z" />
      </svg>
    </span>
  );
}

function FilesIcon() {
  return (
    <span aria-hidden style={{ width: 20, height: 20, display: 'block' }}>
      <svg viewBox="0 0 24 24" fill="currentColor" width="100%" height="100%">
        <path d="M20 6h-8l-2-2H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm0 12H4V8h16v10z" />
      </svg>
    </span>
  );
}

type SidebarProps = {
  /** Which nav item key is active (e.g. 'photos'). Used to show active pill. */
  activeKey?: string;
  navItems?: SidebarNavItem[];
};

export default function Sidebar({
  activeKey,
  navItems = DEFAULT_NAV_ITEMS,
}: SidebarProps) {
  return (
    <div
      className={style({
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: 8,
        paddingBlock: 16,
        width: 64,
        backgroundColor: 'layer-1',
      })}
      style={{
        minHeight: '100%',
        borderRight: '1px solid var(--spectrum-gray-200)',
      }}
    >
      {/* Primary action: add/upload */}
      <Button variant="accent" aria-label="Add" style={{ borderRadius: '50%', width: 40, height: 40, padding: 0 }}>
        <span aria-hidden style={{ fontSize: 20, lineHeight: 1 }}>+</span>
      </Button>

      <Divider size="S" />

      {/* Nav: Home, divider, Photos, Ideas */}
      {navItems.map((item, index) => {
        const isActive = activeKey === item.key;
        return (
          <React.Fragment key={item.key}>
            {index === 1 && <Divider size="S" />}
            <Link
            href={item.href}
            aria-label={item.label}
            aria-current={isActive ? 'page' : undefined}
            UNSAFE_style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: 40,
              height: 40,
              borderRadius: 8,
              position: 'relative',
              ...(isActive
                ? {
                    backgroundColor: 'var(--spectrum-blue-100)',
                    color: 'var(--spectrum-blue-700)',
                  }
                : {}),
            }}
          >
            {item.icon}
            {isActive && (
              <span
                aria-hidden
                style={{
                  position: 'absolute',
                  left: 0,
                  top: '50%',
                  transform: 'translateY(-50%)',
                  width: 3,
                  height: 24,
                  borderRadius: 2,
                  background: 'var(--spectrum-blue-700)',
                }}
              />
            )}
          </Link>
          </React.Fragment>
        );
      })}

      <div className={style({ flex: 1, minHeight: 0 })} />

      {/* Bottom: Files */}
      <Link
        href="/files"
        aria-label="Files"
        UNSAFE_style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          width: 40,
          height: 40,
          borderRadius: 8,
        }}
      >
        <FilesIcon />
      </Link>
    </div>
  );
}
