import { Heading } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';
import Layout from '../components/Layout';

export default function Ideas() {
  return (
    <Layout>
      <div className={style({ display: 'flex', flexDirection: 'column', gap: 16 })}>
        <Heading level={1}>Ideas</Heading>
      </div>
    </Layout>
  );
}
