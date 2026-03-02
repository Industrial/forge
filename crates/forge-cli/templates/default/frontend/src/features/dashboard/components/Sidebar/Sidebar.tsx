import { NavLink } from 'react-router-dom';
import { ActionButton } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };

function DashboardIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <rect x="3" y="3" width="7" height="7" />
      <rect x="14" y="3" width="7" height="7" />
      <rect x="14" y="14" width="7" height="7" />
      <rect x="3" y="14" width="7" height="7" />
    </svg>
  );
}

function OrganizationsIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
      <polyline points="9 22 9 12 15 12 15 22" />
    </svg>
  );
}

function UsersIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
      <circle cx="9" cy="7" r="4" />
      <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
      <path d="M16 3.13a4 4 0 0 1 0 7.75" />
    </svg>
  );
}

function RolesIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
    </svg>
  );
}

function ChevronLeftIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <polyline points="15 18 9 12 15 6" />
    </svg>
  );
}

function ChevronRightIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <polyline points="9 18 15 12 9 6" />
    </svg>
  );
}

const navItems = [
  { to: '/dashboard', label: 'Dashboard', end: true, icon: DashboardIcon },
  { to: '/dashboard/organizations', label: 'Organizations', end: false, icon: OrganizationsIcon },
  { to: '/dashboard/users', label: 'Users', end: false, icon: UsersIcon },
  { to: '/dashboard/roles-and-permissions', label: 'Roles and Permissions', end: false, icon: RolesIcon },
] as const;

const linkClassName = style({
  paddingBlock: 8,
  paddingInline: 12,
  borderRadius: 'default',
  font: 'body',
  textDecoration: 'none',
  cursor: 'pointer',
  display: 'flex',
  alignItems: 'center',
  gap: 12,
});

const iconOnlyClassName = style({
  paddingBlock: 8,
  paddingInline: 12,
  borderRadius: 'default',
  font: 'body',
  textDecoration: 'none',
  cursor: 'pointer',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
});

type SidebarProps = {
  expanded: boolean;
  onToggle: () => void;
};

const asideClassNameExpanded = style({
  width: 240,
  flexShrink: 0,
  borderWidth: 0,
  borderEndWidth: 1,
  borderStyle: 'solid',
  borderColor: 'gray-400',
  backgroundColor: 'layer-1',
  padding: 8,
  display: 'flex',
  flexDirection: 'column',
  gap: 4,
  overflow: 'hidden',
});
const asideClassNameCollapsed = style({
  width: 52,
  flexShrink: 0,
  borderWidth: 0,
  borderEndWidth: 1,
  borderStyle: 'solid',
  borderColor: 'gray-400',
  backgroundColor: 'layer-1',
  padding: 8,
  display: 'flex',
  flexDirection: 'column',
  gap: 4,
  overflow: 'hidden',
});

/* Toggle bar: justifyContent in CSS (macro doesn't accept flex-end/center) */
const toggleBarClassName = style({
  display: 'flex',
  alignItems: 'center',
  paddingBlockEnd: 8,
  borderWidth: 0,
  borderEndWidth: 1,
  borderStyle: 'solid',
  borderColor: 'gray-400',
  marginBlockEnd: 8,
});

export default function Sidebar({ expanded, onToggle }: SidebarProps) {
  const asideClassName = [
    expanded ? asideClassNameExpanded : asideClassNameCollapsed,
    'dashboard-sidebar',
  ].join(' ');

  return (
    <aside
      className={asideClassName}
      aria-label="Dashboard navigation"
    >
      <div
        className={[toggleBarClassName, expanded ? 'dashboard-toggle-bar--expanded' : 'dashboard-toggle-bar--collapsed'].join(' ')}
      >
        <ActionButton
          isQuiet
          aria-label={expanded ? 'Collapse sidebar' : 'Expand sidebar'}
          onPress={onToggle}
        >
          {expanded ? <ChevronLeftIcon /> : <ChevronRightIcon />}
        </ActionButton>
      </div>
      <nav
        className={style({
          display: 'flex',
          flexDirection: 'column',
          gap: 4,
        })}
      >
        {navItems.map(({ to, label, end, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            end={end}
            className={({ isActive }) =>
              [expanded ? linkClassName : iconOnlyClassName, isActive ? 'dashboard-sidebar-link--active' : '']
                .filter(Boolean)
                .join(' ')
            }
            title={!expanded ? label : undefined}
          >
            <Icon />
            {expanded ? <span>{label}</span> : null}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
