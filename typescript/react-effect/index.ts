/**
 * react-effect – Reusable helpers for Effect.ts + React (stream with pending state).
 * Copy or link this package into your app (e.g. under src/lib/react-effect or /data/Code/typescript/react-effect).
 */
export {
  type AsyncState,
  idle,
  pending,
  success,
  failure,
  isIdle,
  isPending,
  isSuccess,
  isFailure,
} from './AsyncState'
export { streamWithPendingState } from './streamWithPendingState'
