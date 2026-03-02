import {
  ActionButton,
  Avatar,
  SearchField,
  Text,
} from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };

type NavbarProps = {
  appName?: string;
  user?: { email: string; name?: string } | null;
};

export default function Navbar({ appName = 'My App', user }: NavbarProps) {
  return (
    <div
      className={style({
        display: 'flex',
        flexDirection: 'row',
        alignItems: 'center',
        gap: 16,
        paddingInline: 16,
        paddingBlock: 12,
        minHeight: 48,
        backgroundColor: 'layer-1',
      })}
      style={{
        borderBottom: '1px solid var(--spectrum-gray-200)',
      }}
    >
      {/* Logo + app name */}
      <div
        className={style({
          display: 'flex',
          flexDirection: 'row',
          alignItems: 'center',
          gap: 8,
          flexShrink: 0,
        })}
      >
        <div
          role="img"
          aria-label="Logo"
          style={{
            width: 28,
            height: 28,
            borderRadius: 6,
            background: 'var(--spectrum-red-600)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'white',
            fontSize: 14,
            fontWeight: 700,
          }}
        >
          A
        </div>
        <Text UNSAFE_style={{ fontWeight: 600 }}>{appName}</Text>
      </div>

      {/* Search - takes remaining space */}
      <div
        className={style({
          flex: 1,
          minWidth: 0,
          maxWidth: 400,
          marginInline: 16,
        })}
      >
        <SearchField
          aria-label="Search photos"
          placeholder="Search photos"
          styles={style({ width: '100%' })}
        />
      </div>

      {/* Utilities: help, app launcher, avatar (no bell per request) */}
      <div
        className={style({
          display: 'flex',
          flexDirection: 'row',
          alignItems: 'center',
          gap: 4,
          flexShrink: 0,
        })}
      >
        <ActionButton aria-label="Help" isQuiet>
          <span aria-hidden style={{ fontSize: 18 }}>?</span>
        </ActionButton>
        <ActionButton aria-label="App launcher" isQuiet>
          <span
            aria-hidden
            style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr 1fr',
              gap: 2,
              width: 18,
              height: 18,
            }}
          >
            {Array.from({ length: 9 }, (_, i) => (
              <span
                key={i}
                style={{
                  width: 4,
                  height: 4,
                  borderRadius: 1,
                  background: 'currentColor',
                }}
              />
            ))}
          </span>
        </ActionButton>
        {user != null ? (
          <Avatar alt={user.name ?? user.email} />
        ) : (
          <div
            style={{
              width: 32,
              height: 32,
              borderRadius: '50%',
              background: 'var(--spectrum-gray-300)',
            }}
          />
        )}
      </div>
    </div>
  );
}
