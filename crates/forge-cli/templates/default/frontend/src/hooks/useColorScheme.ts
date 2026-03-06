import type { PaletteMode } from '@mui/material'
import { useEffect, useState } from 'react'

const STORAGE_KEY = 'mui-color-scheme'

export const useColorSchemeMode = (): [
  PaletteMode,
  React.Dispatch<React.SetStateAction<PaletteMode>>,
] => {
  const [colorSchemeMode, setColorSchemeMode] = useState<PaletteMode>(() => {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (stored === 'light' || stored === 'dark') {
        return stored as PaletteMode
      }
    }
    return 'dark'
  })

  useEffect(() => {
    if (typeof window !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, colorSchemeMode)
    }
  }, [colorSchemeMode])

  return [colorSchemeMode, setColorSchemeMode]
}
