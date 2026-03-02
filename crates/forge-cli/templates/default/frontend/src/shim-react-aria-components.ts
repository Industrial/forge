/**
 * Re-export react-aria-components and alias TableLoadMoreItem as UNSTABLE_TableLoadingIndicator
 * so @react-spectrum/s2@0.8.0 (which expects the old name) works with current react-aria-components.
 */
// 1.10.1 exports UNSTABLE_TableLoadingSentinel; s2@0.8.0 expects UNSTABLE_TableLoadingIndicator
export { UNSTABLE_TableLoadingSentinel as UNSTABLE_TableLoadingIndicator } from 'react-aria-components-table';
export * from 'react-aria-components-original';
