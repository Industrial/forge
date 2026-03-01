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
          <Link href="/login">Login</Link>
          <Link href="/register">Register</Link>
          <Link href="/dashboard">Dashboard</Link>
          <Link href="/photos">Photos</Link>
          <Link href="/ws-demo">WebSocket</Link>
        </div>
      </div>
    </Layout>
  );
}
