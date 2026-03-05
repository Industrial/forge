import Box from '@mui/material/Box'
import CircularProgress from '@mui/material/CircularProgress'

export type LoadingSpinnerProps = {
  /** Vertical padding (default: 4). */
  py?: number
}

export default function LoadingSpinner({ py = 4 }: LoadingSpinnerProps) {
  return (
    <Box sx={{ display: 'flex', justifyContent: 'center', py }}>
      <CircularProgress />
    </Box>
  )
}
