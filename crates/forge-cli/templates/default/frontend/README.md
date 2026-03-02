# Vite + React + Material UI

This is a [Vite](https://vitejs.dev/) React app using [Material UI (MUI)](https://mui.com/) for the UI.

## Getting Started

Install dependencies and run the dev server:

```bash
bun install
bun run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

## Structure

- **App** – `ThemeProvider` (light/dark), `CssBaseline`, and routes.
- **Layout** – Navbar (MUI `AppBar` + `Toolbar`) and main content area for home/profile.
- **Dashboard** – Dashboard layout with Navbar + collapsible Sidebar + content; routes for Dashboard, Organizations, Users, Permissions.
- **Auth** – Login and Register pages with MUI form components.

Theme mode is persisted in `localStorage` under `mui-color-scheme`.
