import { Heading, Link, Text } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';
import Layout from '../components/Layout';

type Props = { locale: string; greeting: string };

export default function Home({ locale, greeting }: Props) {
  return (
    <Layout>
      <div className={style({ display: 'flex', flexDirection: 'column', gap: 16 })}>
        <Heading level={1}>{greeting}</Heading>
        <Text>Locale: {locale}</Text>
        <div className={style({ display: 'flex', flexDirection: 'row', gap: 16, flexWrap: 'wrap' })}>
          <Link href="/login" isQuiet>Login</Link>
          <Link href="/register" isQuiet>Register</Link>
          <Link href="/dashboard" isQuiet>Dashboard</Link>
          <Link href="/photos" isQuiet>Photos</Link>
          <Link href="/ws-demo" isQuiet>WebSocket</Link>
        </div>
      </div>
    </Layout>
  );
}
