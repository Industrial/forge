# react-effect

Effect.ts integration for React. Provides hooks and a runtime that mirror React's built-in hooks (including React 19 patterns) while running logic as Effect programs with typed errors, composition, and Scope-based cleanup.

## Design choices

### 1. Runtime: shared runtime (Option B)

Effects are **not** run ad hoc inside each hook with `Effect.runPromise` in place. Instead, a **React context provides an Effect runtime**. Hooks submit Effects to that runtime and subscribe to their result.

- **Why:** Single place to configure Layers/Services, shared cancellation and resource lifecycle, and consistent behavior across the tree.
- **Implication:** The app (or a subtree) must wrap with `<EffectRuntimeProvider>`. The provider holds a `Runtime<R>` (default or with layers). Hooks consume the runtime via `useEffectRuntime()` and use it to run or fork Effects.

### 2. Pending / success / error state

Async hooks expose a small state machine so success, failure, and “in progress” are explicit and type-safe:

- **Idle** – no run yet (for useActionState-style hooks).
- **Pending** – Effect is running.
- **Success** – Effect succeeded; value is available.
- **Failure** – Effect failed; typed error `E` is available.

So the hook’s “state” is one of `{ _tag: 'idle' } | { _tag: 'pending' } | { _tag: 'success', value: A } | { _tag: 'failure', error: E }`. This aligns with Effect’s `E` and avoids overloading a single `error` slot with `unknown`.

### 3. Cleanup: Scope

Cleanup uses **Scope**, not ad hoc `AbortController` or fiber references only.

- When a hook starts an Effect that should be cancellable (e.g. `useRunEffect`), it:
  1. Allocates a Scope (via the runtime).
  2. Runs the Effect in that Scope (e.g. `Scope.extend(scope, effect)` or equivalent).
  3. On unmount or when deps change, **closes the Scope** (e.g. `Scope.close(scope, Exit.void)`).

- Closing the Scope interrupts any fibers and runs finalizers in that scope, so subscriptions, timers, and other resources are cleaned up in the same way as in pure Effect code.

### 4. API shape: mirror React’s hooks

Hooks are designed to **closely match** the corresponding React hooks, with Effect as the implementation detail:

- **useTransition** → **useTransitionEffect**: `[isPending, startTransition]`; `startTransition` accepts a function that runs an `Effect` (sync or async). Pending is derived from the Effect run.
- **useActionState** → **useActionStateEffect**: `[state, dispatchAction, isPending]`; the “action” is an Effect; state/error/isPending come from the last run.
- **useOptimistic** → **useOptimisticEffect**: `[optimisticState, setOptimistic]`; the “commit” runs an Effect; on success we commit, on failure we revert.
- **useEffect** → **useRunEffect**: `(effect, deps)`; runs the Effect in a Scope and closes the Scope on cleanup.
- **useState** (with Effect setter) → **useEffectState**: like useState, but the setter can accept a value, an updater function, or an `Effect<A, E, R>`; when an Effect is passed, it’s run and the result (or error) drives state.
- **useReducer** (with Effect reducer) → **useEffectReducer**: like useReducer, but the reducer returns `Effect<NextState, E, R>`; dispatch runs that Effect and updates state from the result.

So the **mental model** for React developers is “same hooks, but the async/effectful part is an Effect.” We do not invent a parallel API; we keep names and signatures close and document the small differences (e.g. action is `() => Effect<...>` or `(prev, payload) => Effect<...>`).

### 5. useOptimistic

Optimistic state is “show X immediately, then run an Effect; on success commit, on failure revert.” The hook manages:

- Committed state (last confirmed value).
- Optimistic state (what the UI shows).
- In-flight Effect; on success we commit the optimistic value (or the result), on failure we revert to the previous committed state.

The “action” is expressed as an Effect so success/failure and typing are consistent with the rest of the stack.

### 6. useState / useReducer

- **useEffectState**: setter can be called with `A`, `(prev: A) => A`, or `Effect<A, E, R>`. When an Effect is passed, the hook runs it (via the runtime), then sets state to the success value or exposes the error (e.g. in a separate error state or in the same state machine).
- **useEffectReducer**: reducer has shape `(state, action) => Effect<NextState, E, R>`. Dispatch runs that Effect and updates state from the result; error and isPending can be exposed the same way as in useActionStateEffect.

Request ordering / cancellation (e.g. “cancel previous when dispatch is called again”) can be added on top of this by having the runtime or the hook track the current run and interrupt or ignore stale completions.

## File layout

- **README.md** (this file) – design choices and usage.
- **AsyncState.ts** – `AsyncState<A, E>` and helpers: `idle`, `pending`, `success`, `failure`, `isIdle`, `isPending`, `isSuccess`, `isFailure`.
- **EffectRuntime.tsx** – React context for the Effect runtime and `EffectRuntimeProvider` / `useEffectRuntime`.
- **useRunEffect.ts** – run an Effect when deps change; cleanup by closing the Scope.
- **useTransitionEffect.ts** – `[isPending, startTransition]`; startTransition runs an Effect.
- **useActionStateEffect.ts** – `[state, dispatchAction, isPending]`; action is an Effect.
- **useOptimisticEffect.ts** – `[optimisticState, setOptimistic]`; commit/revert via Effect.
- **useEffectState.ts** – useState with setter that can take a value, updater, or Effect.
- **useEffectReducer.ts** – useReducer with reducer that returns an Effect.
- **index.ts** – re-exports.

## Usage

1. Wrap the app (or subtree) with `EffectRuntimeProvider`. Optionally pass a custom `Runtime<R>` or build one from Layers.
2. Use the hooks inside that tree: `useEffectRuntime()` for the runtime, and the effect hooks as needed.
3. For hooks that need `R` (e.g. `HttpClient`), ensure the provider’s runtime is built with the required layers so that `Effect<A, E, R>` can be run.

## Dependencies

- `effect` – Effect core (Runtime, Scope, Effect, Fiber, etc.).
- `react` – for context, hooks, and types.

No React 19 requirement; the API is designed so that when you upgrade to React 19, the same patterns (useTransition, useActionState, useOptimistic) align with the built-in hooks.
