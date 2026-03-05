import { useState, useEffect } from 'react'
import { useLocation, Outlet } from 'react-router-dom'
import Box from '@mui/material/Box'
import Drawer from '@mui/material/Drawer'
import useMediaQuery from '@mui/material/useMediaQuery'
import { useTheme } from '@mui/material/styles'
import Sidebar from '../../components/Sidebar/Sidebar'
import Navbar from '../../../../components/Navbar'
export type DashboardLayoutProps = {
  children: React.ReactNode
  colorScheme: 'light' | 'dark'
  onToggleTheme: () => void
}

export default function DashboardLayout({
  children: _children,
  colorScheme,
  onToggleTheme,
}: DashboardLayoutProps) {
  const theme = useTheme()
  const isDesktop = useMediaQuery(theme.breakpoints.up('md'))
  const location = useLocation()
  const [sidebarExpanded, setSidebarExpanded] = useState(true)
  const [mobileOpen, setMobileOpen] = useState(false)

  useEffect(() => {
    setMobileOpen(false)
  }, [location.pathname])

  return (
    <>
      <Navbar
        appName="App"
        colorScheme={colorScheme}
        onToggleTheme={onToggleTheme}
        onOpenSidebar={isDesktop ? undefined : () => setMobileOpen(true)}
      />
      <Box
        className="dashboard-page"
        sx={{
          display: 'flex',
          flexDirection: 'row',
          flexGrow: 1,
          minHeight: 0,
          overflow: 'hidden',
        }}
      >
        {isDesktop ? (
          <Sidebar
            expanded={sidebarExpanded}
            onToggle={() => setSidebarExpanded((e) => !e)}
          />
        ) : (
          <Drawer
            variant="temporary"
            anchor="left"
            open={mobileOpen}
            onClose={() => setMobileOpen(false)}
            slotProps={{
              paper: {
                sx: {
                  width: '80%',
                  boxSizing: 'border-box',
                  mt: 0,
                  pt: 0,
                },
              },
            }}
          >
            <Sidebar
              expanded
              onToggle={() => {}}
              hideToggle
              disableBorder
              fullWidth
            />
          </Drawer>
        )}
        <Box
          className="dashboard-content"
          sx={{
            flexGrow: 1,
            minWidth: 0,
            overflow: 'auto',
            py: 2,
            px: 2,
          }}
        >
          <Outlet />
        </Box>
      </Box>
    </>
  )
}
