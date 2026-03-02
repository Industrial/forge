import { Heading } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React from 'react';

export default function Ideas() {
  return (
    <div className={style({ display: 'flex', flexDirection: 'column', gap: 16 })}>
      <Heading level={1}>Ideas</Heading>
    </div>
  );
}
