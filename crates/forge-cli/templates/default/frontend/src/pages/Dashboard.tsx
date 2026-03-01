import React from 'react';
import Layout from '../components/Layout';

type Props = { message: string };

export default function Dashboard({ message }: Props) {
  return (
    <Layout>
      <div>
        <h1>Dashboard</h1>
        <p>{message}</p>
        <nav>
          <a href="/">Home</a> | <a href="/login">Login</a> | <a href="/ws-demo">WebSocket</a>
        </nav>
      </div>
    </Layout>
  );
}
