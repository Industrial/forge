# Why `useRunEffect` might not fire on mount

## Source: `react-effect-hooks`

From `node_modules/react-effect-hooks/dist/useRunEffect.js`:

```javascript
export function useRunEffect(effect, deps, runtime) {
    const resolvedRuntime = runtime !== undefined ? runtime : useEffectRuntime().runtime;
    const scopeRef = useRef(null);
    const isMountedRef = useRef(true);
    useEffect(() => {
        if (resolvedRuntime === null)
            return;
        // ... runs Effect in a Scope, Runtime.runPromise(resolvedRuntime)(scoped)
        run();
        return () => { /* cleanup: close scope */ };
    }, [resolvedRuntime, ...deps]);
}
```

- **Signature:** `useRunEffect(effect, deps, runtime?)`
- **Behavior:** Same as `useEffect`: runs on mount and when any dependency changes. Dependencies are `[resolvedRuntime, ...deps]`.
- **When it does not run:** Only when `resolvedRuntime === null` (early return). It does **not** skip when `runtime` is `undefined` (then it uses context runtime).

So under normal conditions, **`useRunEffect` does run on mount** whenever `resolvedRuntime` is not `null`.

## How `resolvedRuntime` is set

- **Third argument passed:** `resolvedRuntime = runtime` (so if you pass `null`, the effect never runs).
- **Third argument omitted:** `resolvedRuntime = useEffectRuntime().runtime` (from `EffectRuntimeProvider` context).

So the effect is skipped only when the **resolved** runtime is explicitly `null`.

## Why it might not fire for OrganizationsPage

1. **`runtime === null` when the page first renders**  
   If `useEffectRuntime().runtime` is still `null` on the first render where OrganizationsPage mounts (e.g. provider not committed yet, or a different tree), then passing that `runtime` into `useRunEffect(..., runtime)` makes `resolvedRuntime === null`, so the effect returns without running.

2. **OrganizationsPage never mounts**  
   If `DashboardPermissionGuard` never renders its children, the list effect never runs:
   - **`loading === true`** → guard returns `null` → OrganizationsPage is not mounted.
   - **`!hasPermission(permissions, permission)`** → guard does `<Navigate to="/dashboard" />` → user is redirected, OrganizationsPage never mounts.

So “useRunEffect isn’t firing on mount” can be either:
- **Effect not run:** `resolvedRuntime` is `null` when the hook’s `useEffect` runs, or
- **Component not mounted:** the guard keeps returning `null` or redirecting, so the hook never runs at all.

## How to confirm

1. **Confirm the page mounts**  
   On `/dashboard/organizations`, do you see the Organizations UI (e.g. empty list, filters, “Add organization” if permitted), or are you redirected to `/dashboard`?  
   - If redirected → fix permissions / guard so the page can mount.  
   - If the page is visible → the component mounts; then the issue is (1) above.

2. **Check `resolvedRuntime` on mount**  
   In `OrganizationsPage`, temporarily log right before `useRunEffect`:
   - `console.log('OrganizationsPage mount', { runtime: runtime != null, runtimeNull: runtime === null })`
   and inside a small effect that runs once:
   - Log inside `useRunEffect`’s effect (e.g. in a wrapper) to see if the inner `useEffect` runs and whether it hits the `if (resolvedRuntime === null) return` path.  
   If the inner effect runs and `resolvedRuntime` is non-null, then the Effect is being run and the problem is elsewhere (e.g. the Effect itself or the HTTP/client layer). If the inner effect never runs or always sees `resolvedRuntime === null`, then the fix is to ensure a non-null runtime is available when OrganizationsPage first mounts (e.g. only render the page when `runtime != null`, or don’t pass `runtime` so the hook uses context runtime).

## Summary

- **Library behavior:** `useRunEffect` is designed to run on mount and when `[resolvedRuntime, ...deps]` change; it only skips when `resolvedRuntime === null`.
- **Likely causes when it “doesn’t fire on mount”:** (1) `resolvedRuntime` is `null` (e.g. passing through a null `runtime` from context), or (2) the component that calls `useRunEffect` never mounts because the guard returns `null` or redirects.
