import Box from '@mui/material/Box'
import LoadingSpinner from './LoadingSpinner'

/**
 * Centered loading indicator for route guards and full-bleed areas.
 * Uses MUI Box + LoadingSpinner (use when ThemeProvider is mounted).
 */
export default function CenteredLoader(): React.JSX.Element {
  return (
    <Box
      sx={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        flex: 1,
        minHeight: '40vh',
      }}
      aria-busy
      aria-label="Loading"
    >
      <LoadingSpinner />
    </Box>
  )
}
