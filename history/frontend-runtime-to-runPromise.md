# Migrating frontend from React runtime context to Effect.runPromise / single app runtime

## Goal

Remove the React-based runtime plumbing (`EffectRuntimeProvider`, `useEffectRuntime()`, passing `runtime` through context and as third arg to `useRunEffect`). Have a single app runtime stored in a module and run all effects via `runApp(effect)` (which uses `Runtime.runPromise(appRuntime)(effect)` under the hood). Components no longer depend on "runtime from context" or `react-effect-hooks` for running app effects.

## Why

- Simpler mental model: one runtime, one way to run effects (`runApp`).
- No "runtime not ready" / "runtime null" issues in children: either the app shows FullPageLoader (runtime not ready) or children mount and `runApp` always has the runtime.
- No need for `useRunEffect(effect, deps, runtime)` or `useEffectRuntime()` in every component.
- Fewer dependencies on `react-effect-hooks` (we can keep it for `useEffectState`, `streamWithPendingState`, `runStreamInto` if we want, or replace those too later).

## What stays the same

- We still build **one** Effect runtime with `AppLayer` (same as now).
- We still build it in **AuthenticationRuntimeProvider** (after config/baseUrl is known), run rehydrate, then consider the app "ready."
- We still show **FullPageLoader** until the runtime is built and rehydrate has run; then we render children. So by the time any dashboard/auth page mounts, the runtime exists.
- **Effect.runPromise** still requires a runtime for effects that have requirements (e.g. `EntityApi`, `HttpClient`). We don't remove the runtime; we stop passing it through React and instead hold it in a module.

## What changes

### 1. App runtime module (new)

**File:** `lib/appRuntime.ts` (or `lib/runApp.ts`)

- Hold the runtime in a module-level variable (or ref):
  - `let appRuntime: Runtime.Runtime<AppServices> | null = null`
- Export:
  - `setAppRuntime(r: Runtime.Runtime<AppServices> | null): void` — called by AuthenticationRuntimeProvider when the runtime is built (or cleared).
  - `getAppRuntime(): Runtime.Runtime<AppServices> | null`
  - `runApp<A, E, R extends AppServices>(effect: Effect.Effect<A, E, R>): Promise<A>` — same as current `runWithAppRuntime`, but uses `getAppRuntime()` internally. If runtime is null, returns `Promise.reject(new Error('App runtime not ready'))`.
- Optionally: `onRuntimeReady: Promise<Runtime.Runtime<AppServices>>` or a small "when ready" helper if any code ever needed to wait for the runtime before the tree mounts (probably not needed if we keep FullPageLoader until ready).

### 2. AuthenticationRuntimeProvider

- Keep building the layer and runtime in `useEffect` (same as now).
- When the runtime is ready: call **`setAppRuntime(r)`** in addition to (or instead of) `setRuntime(r)`.
- **Stop rendering** `<EffectRuntimeProvider runtime={runtime}>{children}</EffectRuntimeProvider>`. Render just `{children}` when ready (we still need React state to know "ready" so we can hide FullPageLoader and show children).
- So: keep `const [runtime, setRuntime] = useState<...>(null)` only to know when we're ready (setRuntime(r) when built). Optionally you can drop `runtime` state and use a boolean `isReady` that gets set when we call `setAppRuntime(r)`, so the only storage is the module.
- **useRunEffect(connectEffect, [runtime, config.token], runtime)** for WebSocket connect: replace with **useEffect** that calls **runApp(connectEffect)** when ready (e.g. when `getAppRuntime()` is non-null). So we need a way to re-render when runtime becomes ready — e.g. keep `setRuntime(r)` or `setReady(true)` so the provider re-renders and then this effect runs with runApp. So we still have one piece of state "ready" or "runtime" to trigger showing children and running the connect effect.

### 3. Remove all `useEffectRuntime()` and `runWithAppRuntime(runtime, effect)`

Replace with `runApp(effect)` everywhere. No more passing `runtime` as an argument.

**Files to update (from search):**

- `lib/appLayer.ts` — keep `runWithAppRuntime` for backward compat or remove it and only export `runApp` from `appRuntime.ts`. Prefer: add `appRuntime.ts` with `runApp` that uses `getAppRuntime()`; then replace all `runWithAppRuntime(runtime, effect)` with `runApp(effect)` and remove `runWithAppRuntime` from appLayer (or re-export runApp from appLayer).
- `lib/AuthenticationRuntimeProvider.tsx` — as above; stop using EffectRuntimeProvider; use setAppRuntime; use useEffect + runApp for connect effect.
- `context/AuthenticationContext.tsx` — remove `useEffectRuntime`, use `runApp(...)` instead of `runWithAppRuntime(runtime, ...)`.
- `context/ForgeWebsocketContext.tsx` — same.
- `features/authentication/hooks/useAuthentication.ts` — use `runApp(getStateEffect)`; no `useEffectRuntime`. Need to handle "runtime not ready": when would this hook run before runtime is ready? Only if the component using it is mounted before the provider shows children. So as long as we only render the tree that uses useAuthentication after we're ready, we're fine. So no change to "when" it runs.
- `features/authentication/pages/LoginPage.tsx`, `RegisterPage.tsx`, `SelectScopePage.tsx` — remove useEffectRuntime, use runApp.
- `features/dashboard/components/DashboardScopeGuard.tsx` — remove useEffectRuntime, use runApp.
- `features/dashboard/pages/OrganizationsPage.tsx`, `UsersPage.tsx`, `RolesPage.tsx`, `PermissionsPage.tsx`, `AuditLogPage.tsx` — remove useEffectRuntime; replace useRunEffect(effect, deps, runtime) with useEffect(() => { runApp(effect) }, deps); replace runWithAppRuntime(runtime, effect) with runApp(effect).
- `components/GenericEntityCrud.tsx` — same pattern.
- `components/SubscriptionStreamRunner.tsx` — remove useEffectRuntime, use runApp; useEffect(..., [token]) (no runtime in deps).
- `hooks/useEntitySubscription.ts` — remove useEffectRuntime; use runApp; deps without runtime.

### 4. Replace `useRunEffect(effect, deps, runtime)` with `useEffect` + `runApp`

Pattern:

```ts
// Before
useRunEffect(refreshEffect, [liveRefreshTrigger, setListStateAsEffect], runtime)

// After
useEffect(() => {
  runApp(refreshEffect)
}, [liveRefreshTrigger, setListStateAsEffect])
```

Note: `refreshEffect` is recreated every render. If we want to avoid exhaustive-deps warnings and unnecessary reruns, we can memoize the effect or rely on deps (e.g. only run when liveRefreshTrigger or other deps change). Same as current useRunEffect behavior.

For "run once on mount" we already have in OrganizationsPage:

```ts
useEffect(() => {
  if (!runtime) return
  runWithAppRuntime(runtime, refreshEffect)
}, [runtime])
```

After migration:

```ts
useEffect(() => {
  runApp(refreshEffect)
}, []) // or [liveRefreshTrigger] if we want to re-run on trigger
```

No need to check `runtime` because we don't render this tree until runtime is set.

### 5. Optional: remove dependency on `react-effect-hooks` for running effects

- We can keep using `useEffectState`, `streamWithPendingState`, `runStreamInto` from react-effect-hooks if they don't require the runtime from context. Check: do they use useEffectRuntime internally? If yes, we'd need to refactor those too or keep a minimal EffectRuntimeProvider that only provides the runtime (so those hooks still work). Looking at the code, useRunEffect is the one that takes runtime; useEffectState and runStreamInto might not need context. So we might be able to remove EffectRuntimeProvider entirely and only use useEffect + runApp for "run effect when deps change."
- If we remove EffectRuntimeProvider completely, any hook from react-effect-hooks that calls useEffectRuntime() will throw. So we need to avoid those. From the search, the only usages of useEffectRuntime are to get runtime and then call runWithAppRuntime or pass to useRunEffect. So once we replace all of those with runApp and useEffect, we don't need useEffectRuntime. And we're not using useRunEffect with context runtime anymore (we're using useEffect + runApp). So we can remove EffectRuntimeProvider. We might still use useRunEffect from react-effect-hooks with no third argument: useRunEffect(effect, deps). Then the hook would use useEffectRuntime().runtime. So it would throw because there's no provider. So we must replace every useRunEffect with useEffect + runApp. Then we can remove EffectRuntimeProvider and the dependency on react-effect-hooks for runtime/useRunEffect. We might still keep the package for useEffectState, runStreamInto, streamWithPendingState if we use them.

### 6. Types

- `AppServices` stays in appLayer (or move to appRuntime.ts).
- `runApp` is typed as `runApp<A, E, R extends AppServices>(effect: Effect.Effect<A, E, R>): Promise<A>`.

## Summary checklist

1. Add **appRuntime.ts**: `setAppRuntime`, `getAppRuntime`, `runApp`.
2. **AuthenticationRuntimeProvider**: build runtime → `setAppRuntime(r)`, show children when ready; **do not** render EffectRuntimeProvider; run WebSocket connect via `useEffect` + `runApp(connectEffect)`.
3. **Replace everywhere**: `runWithAppRuntime(runtime, effect)` → `runApp(effect)`; remove `useEffectRuntime()` and `runtime` from component code.
4. **Replace every useRunEffect** that runs an app effect with `useEffect(() => { runApp(effect).catch(...) }, deps)`.
5. Remove **EffectRuntimeProvider** from the tree.
6. Optionally remove **react-effect-hooks** from package.json if we no longer use any of its hooks; otherwise keep it only for useEffectState/runStreamInto/streamWithPendingState if they don't require context.

## Risks / notes

- **Concurrent React**: If in the future we have components that mount before the runtime is ready (e.g. a route that doesn't go through the provider), they must not call runApp until ready or runApp will reject. Keeping the current "FullPageLoader until ready" pattern avoids this.
- **Testing**: Tests that mock the runtime or run effects need to call `setAppRuntime(mockRuntime)` before rendering components that use runApp.
- **WebsocketLive / ForgeWebsocketLive**: They use `Effect.runtime<never>()` and `Runtime.runPromise(runtime)(effect)` internally. That's inside an Effect; the runtime there is the one running the effect (the app runtime). So when we run the app with our single runtime, those services still get that runtime. No change needed there.
