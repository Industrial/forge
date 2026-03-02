import { Navigate } from 'react-router-dom';
import { useSession } from '../context/Session';

export default function Dashboard() {
  const { user } = useSession();

  if (user == null) {
    return <Navigate to="/login" replace />;
  }

  return (
    <div>
      <h1>Dashboard</h1>
      <p>Welcome to the dashboard.</p>
      <nav>
        <a href="/">Home</a> | <a href="/login">Login</a> | <a href="/ws-demo">WebSocket</a>
      </nav>
    </div>
  );
}
