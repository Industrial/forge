import { useState } from 'react';
import { Outlet } from 'react-router-dom';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import Sidebar from '../../components/Sidebar/Sidebar';

export default function DashboardLayout() {
  const [sidebarExpanded, setSidebarExpanded] = useState(true);

  return (
    <div
      className={[style({
        display: 'flex',
        flexDirection: 'row',
        flexGrow: 1,
        minHeight: 0,
        overflow: 'hidden',
      }), 'dashboard-page'].join(' ')}
    >
      <Sidebar
        expanded={sidebarExpanded}
        onToggle={() => setSidebarExpanded((e) => !e)}
      />
      <div
        className={[style({
          flexGrow: 1,
          minWidth: 0,
          overflow: 'auto',
          paddingBlock: 16,
          paddingInline: 16,
        }), 'dashboard-content'].join(' ')}
      >
        <Outlet />
      </div>
    </div>
  );
}
