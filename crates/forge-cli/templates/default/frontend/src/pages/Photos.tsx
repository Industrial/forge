import { ActionButton, Heading } from '@react-spectrum/s2';
import { style } from '@react-spectrum/s2/style' with { type: 'macro' };
import React, { useState } from 'react';

const PLACEHOLDER_IMAGES = [
  'https://picsum.photos/seed/1/400/300',
  'https://picsum.photos/seed/2/400/400',
  'https://picsum.photos/seed/3/400/500',
  'https://picsum.photos/seed/4/400/300',
  'https://picsum.photos/seed/5/400/350',
  'https://picsum.photos/seed/6/400/450',
  'https://picsum.photos/seed/7/400/300',
  'https://picsum.photos/seed/8/400/400',
  'https://picsum.photos/seed/9/400/500',
  'https://picsum.photos/seed/10/400/300',
  'https://picsum.photos/seed/11/400/350',
  'https://picsum.photos/seed/12/400/450',
];

type GridSize = 2 | 3;

export default function Photos() {
  const [gridSize, setGridSize] = useState<GridSize>(2);

  return (
    <div className={style({ display: 'flex', flexDirection: 'column', gap: 24 })}>
        <div
          className={style({
            display: 'flex',
            flexDirection: 'row',
            justifyContent: 'space-between',
            alignItems: 'center',
            flexWrap: 'wrap',
          })}
        >
          <Heading level={1}>Photos</Heading>
          <div className={style({ display: 'flex', flexDirection: 'row', gap: 4 })}>
            <ActionButton
              aria-label="Grid 2 by 2"
              isQuiet
              // isSelected={gridSize === 2}
              onPress={() => setGridSize(2)}
            >
              <Grid2Icon />
            </ActionButton>
            <ActionButton
              aria-label="Grid 3 by 3"
              isQuiet
              // isSelected={gridSize === 3}
              onPress={() => setGridSize(3)}
            >
              <Grid3Icon />
            </ActionButton>
          </div>
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: `repeat(${gridSize}, minmax(0, 1fr))`,
            gap: 12,
          }}
        >
          {PLACEHOLDER_IMAGES.map((src, i) => (
            <div
              key={i}
              style={{
                aspectRatio: '4/3',
                borderRadius: 8,
                overflow: 'hidden',
                background: 'var(--spectrum-gray-200)',
              }}
            >
              <img
                src={src}
                alt=""
                style={{
                  width: '100%',
                  height: '100%',
                  objectFit: 'cover',
                }}
              />
            </div>
          ))}
        </div>
      </div>
  );
}

function Grid2Icon() {
  return (
    <span aria-hidden style={{ width: 20, height: 20, display: 'block' }}>
      <svg viewBox="0 0 24 24" fill="currentColor" width="100%" height="100%">
        <path d="M3 3h8v8H3V3zm10 0h8v8h-8V3zM3 13h8v8H3v-8zm10 0h8v8h-8v-8z" />
      </svg>
    </span>
  );
}

function Grid3Icon() {
  return (
    <span aria-hidden style={{ width: 20, height: 20, display: 'block' }}>
      <svg viewBox="0 0 24 24" fill="currentColor" width="100%" height="100%">
        <path d="M3 3h5v5H3V3zm6 0h5v5H9V3zm6 0h5v5h-5V3zM3 9h5v5H3V9zm6 0h5v5H9V9zm6 0h5v5h-5V9zM3 15h5v5H3v-5zm6 0h5v5H9v-5zm6 0h5v5h-5v-5z" />
      </svg>
    </span>
  );
}
