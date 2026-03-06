export const getBaseUrl = () => {
  // TODO: Get this from an env var? How do we do this in Vite?
  if (typeof window === 'undefined') {
    return 'http://localhost:5173'
  }

  return window.location.origin
}
