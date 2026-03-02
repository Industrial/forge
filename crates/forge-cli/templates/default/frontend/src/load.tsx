// This file is used to load css files into the browser.
import React from 'react';

import '@react-spectrum/s2/page.css';

export type LoadProps = {
  children: React.ReactNode;
};

export function Load({ children }: LoadProps) {
  return (
    <>
      {children}
    </>
  );
}