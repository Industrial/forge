import useTheme from '@mui/material/styles/useTheme'
import useMediaQuery from '@mui/material/useMediaQuery'

type Breakpoint = 'xs' | 'sm' | 'md' | 'lg' | 'xl'

/**
 * Returns true when viewport is below the given breakpoint (default 'md').
 * Useful for switching between mobile and desktop layouts.
 */
export function useIsMobile(breakpoint: Breakpoint = 'md'): boolean {
  const theme = useTheme()
  return useMediaQuery(theme.breakpoints.down(breakpoint))
}
