import Box from '@mui/material/Box'
import CircularProgress from '@mui/material/CircularProgress'
import { useComponentLogger } from '@/hooks'

export type LoadingSpinnerProps = {
  /** Vertical padding (default: 4). */
  py?: number
}

export default function LoadingSpinner({ py = 4 }: LoadingSpinnerProps) {
  useComponentLogger('LoadingSpinner')
  return (
    <Box sx={{ display: 'flex', justifyContent: 'center', py }}>
      <CircularProgress />
    </Box>
  )
}
