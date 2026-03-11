# forge-react-effect-store

Effect.ts-style reactive store for React: in-memory state with `Layer.sync`, and React hooks for subscription.

## Usage

- **`defineStore(name, initial)`** / **`makeReactiveStore`** — Create a store; returns `{ tag, layer }`. Add `layer` to your app Effect layer; use `tag` in effects and with the hooks.
- **`useReactiveStore(tag, initial, run, runFork)`** — Subscribe to the store in React; returns current value.
- **`useReactiveStoreWithInit(tag, initial, run, runFork)`** — Same but returns `{ value, initialized }`.
- **`useDerivedReactiveStore(tag, initialB, run, runFork, derivationKey, derive)`** — Subscribe to a derived stream (e.g. `pipe(store.changes, Stream.map(...))`).
- **`clearReactiveStoreCacheForTesting(tag)`** — Test-only: clear the cached external store for a tag.

Requires `effect` and `react` as peer dependencies. Your app must provide `run` and `runFork` (e.g. from a hook that provides the app layer).
