
const containerStyle: React.CSSProperties = {
  display: 'flex',
  justifyContent: 'center',
  alignItems: 'center',
  height: '100vh',
  width: '100%',
}

const spinnerStyle: React.CSSProperties = {
  width: 40,
  height: 40,
  border: '3px solid rgba(0,0,0,0.1)',
  borderTopColor: '#1976d2',
  borderRadius: '50%',
  animation: 'fullPageLoaderSpin 0.8s linear infinite',
}

/** Full-page loading indicator. Inline styles only (no MUI) for use before ThemeProvider. */
export default function FullPageLoader(): React.JSX.Element {
  return (
    <div style={containerStyle} aria-busy aria-label="Loading">
      <div style={spinnerStyle} />
      <style>{`@keyframes fullPageLoaderSpin { to { transform: rotate(360deg); } }`}</style>
    </div>
  )
}
