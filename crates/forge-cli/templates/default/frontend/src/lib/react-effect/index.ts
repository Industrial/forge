/**
 * react-effect – Effect.ts integration for React.
 *
 * Provides hooks and a shared runtime that mirror React's built-in hooks
 * (including React 19 patterns like useTransition, useActionState, useOptimistic)
 * while running logic as Effect programs with typed errors and Scope-based cleanup.
 *
 * @packageDocumentation
 */

export {
  EffectRuntimeProvider,
  useEffectRuntime,
  type EffectRuntimeContext,
  type EffectRuntimeProviderProps,
} from './EffectRuntime'
export { useRunEffect } from './useRunEffect'
export { useTransitionEffect } from './useTransitionEffect'
export { useActionStateEffect } from './useActionStateEffect'
export { useOptimisticEffect } from './useOptimisticEffect'
export {
  useEffectState,
  type SetStateAsEffect,
} from './useEffectState'
export { useEffectReducer } from './useEffectReducer'
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
export { runStreamInto } from './runStreamInto'
