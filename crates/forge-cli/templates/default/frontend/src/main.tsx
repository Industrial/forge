import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'

import '@/reset.css'
import App from '@/App.tsx'

const rootElement = document.getElementById('root')
if (!rootElement) {
  throw new Error('Failed to find the root element')
}

console.log('main')

ReactDOM.createRoot(rootElement).render(
  <BrowserRouter>
    <App />
  </BrowserRouter>,
)
