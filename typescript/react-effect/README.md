# react-effect (standalone copy)

Reusable Effect.ts + React helpers. This folder is a **copy** of the pattern used in `crates/forge-cli/templates/default/frontend/src/lib/react-effect/`.

To use this in a separate TypeScript project (e.g. at `/data/Code/typescript/react-effect`):

1. Copy this folder to your project, or
2. Add `effect` as a dependency and import from here.

**Exports:**

- `AsyncState<A, E>`, `idle`, `pending`, `success`, `failure`, `isIdle`, `isPending`, `isSuccess`, `isFailure`
- `streamWithPendingState(effect)` – builds a Stream that emits `pending` then `success(a)` or `failure(e)` for the given effect; use with `Stream.runForEach(stream, setStateAsEffect)` to drive React state.
