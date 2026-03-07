# Reactive Store Library Redesign — Swarm Brief for Claude Code

Use this as the single input to parallelize and consume the epic **forge-z628** as fast as possible. No CLI lookups needed: all IDs, commands, and implementation hints are below.

---

## Rules

- **Shell:** Wrap every command in `devenv shell --`, e.g. `devenv shell -- bd update forge-z628.1 --status in_progress --claim --json`.
- **bd:** Use bd for all task state. After implementing: `devenv shell -- bd close <id> --reason "Done" --json`. Commit `.beads/issues.jsonl` with the code change.
- **Parallelism:** Work on all tickets in the same wave at once (e.g. spawn one agent per ticket in Wave 1). Only start a ticket when all its dependencies (listed below) are closed.

---

## Epic and swarm

- **Epic:** forge-z628 — Reactive Store Library Redesign: automagic wiring, Layer.sync, no main plumbing
- **Plan doc:** `history/reactive-store-library-redesign.md` (architecture, API, mermaid)
- **Swarm ID:** forge-8tpz (created from epic)

---

## Dependency graph (waves)

- **Wave 1 (parallel, no deps):** forge-z628.1, forge-z628.2, forge-z628.3  
- **Wave 2 (after 1):** forge-z628.4, forge-z628.5 (both depend on .1)  
- **Wave 3 (after 2):** forge-z628.6 (depends on .4, .5)  
- **Wave 4 (after 3):** forge-z628.7 (depends on .2, .3), forge-z628.9 (depends on .6), forge-z628.11 (depends on .6)  
- **Wave 5 (after 4):** forge-z628.8 (depends on .7), forge-z628.10 (depends on .9)  
- **Wave 6 (after 5):** forge-z628.12 (depends on .7, .8)

So: run 1,2,3 in parallel → then 4,5 in parallel → then 6 → then 7,9,11 in parallel → then 8,10 in parallel → then 12.

---

## Tickets (copy-paste ready)

### forge-z628.1 — [ReactiveStore] Add mutable state (current, registry, changeListeners) in makeReactiveStore
- **File:** `crates/forge-cli/templates/default/frontend/src/lib/ReactiveStore.ts`
- **Do:** Inside `makeReactiveStore`, add `let current: A = initial`, keep existing `registry`, add `const changeListeners = new Set<(a: A) => void>()`. Do not remove `Layer.scoped` yet; just introduce the variables and wire them where the current ref is used (or leave ref for now and only add the new state).
- **Claim:** `devenv shell -- bd update forge-z628.1 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.1 --reason "Done" --json`

### forge-z628.2 — [appLayer] Remove authStoreOverride parameter from buildApplicationLayer
- **File:** `crates/forge-cli/templates/default/frontend/src/lib/appLayer.ts`
- **Do:** Remove the `authStoreOverride?: AuthenticationStateReactiveStore` parameter. Always set `authStoreLayer = getAuthenticationStateStoreLayer()`. Remove the `Layer.succeed(AuthenticationStateReactiveStoreTag, authStoreOverride)` branch and the import of `AuthenticationStateReactiveStoreTag` if only used there.
- **Claim:** `devenv shell -- bd update forge-z628.2 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.2 --reason "Done" --json`

### forge-z628.3 — [appLayer] Remove setApplicationLayer and singleton setter
- **File:** `crates/forge-cli/templates/default/frontend/src/lib/appLayer.ts`
- **Do:** Delete the function `setApplicationLayer` and its JSDoc. Ensure `getApplicationLayer()` only builds the layer on first call (no setter).
- **Claim:** `devenv shell -- bd update forge-z628.3 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.3 --reason "Done" --json`

### forge-z628.4 — [ReactiveStore] Implement get/update/notify on in-memory state (store shape)
- **File:** `crates/forge-cli/templates/default/frontend/src/lib/ReactiveStore.ts`
- **Do:** Build a `store` object: `get: () => Effect.succeed(current)`, `update: (f) => Effect.sync(() => { const next = f(current); current = next; registry.setter?.(next); changeListeners.forEach(l => l(next)); })`. Expose a single `notify(next)` if helpful. Use this object later for Layer.sync; for this ticket you can still keep Layer.scoped that returns this store (or temporarily return this from the scoped effect). Depends on .1 (current, registry, changeListeners exist).
- **Claim:** `devenv shell -- bd update forge-z628.4 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.4 --reason "Done" --json`

### forge-z628.5 — [ReactiveStore] Implement changes stream from changeListeners (Stream.async)
- **File:** `crates/forge-cli/templates/default/frontend/src/lib/ReactiveStore.ts`
- **Do:** Implement `store.changes` as an Effect Stream that (1) emits `current` once, (2) subscribes to `changeListeners` so each update is emitted, (3) returns cleanup that removes the listener. Use `Stream.async` or equivalent (e.g. `Stream.async<A>((emit) => { emit(Effect.succeed(current)); const l = (a: A) => { emit(Effect.succeed(a)) }; changeListeners.add(l); return () => { changeListeners.delete(l) }; })`). Depends on .1.
- **Claim:** `devenv shell -- bd update forge-z628.5 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.5 --reason "Done" --json`

### forge-z628.6 — [ReactiveStore] Replace Layer.scoped with Layer.sync in makeReactiveStore
- **File:** `crates/forge-cli/templates/default/frontend/src/lib/ReactiveStore.ts`
- **Do:** Remove `SubscriptionRef.make` and `Layer.scoped`. Use `Layer.sync(tag, () => store)` where `store` is the object with `get`, `update`, `changes` (from .4 and .5). Remove SubscriptionRef import if unused.
- **Claim:** `devenv shell -- bd update forge-z628.6 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.6 --reason "Done" --json`

### forge-z628.7 — [main] Revert main.tsx to simple bootstrap
- **File:** `crates/forge-cli/templates/default/frontend/src/main.tsx`
- **Do:** Replace the current `Effect.scoped` bootstrap with: get layer via `getApplicationLayer()`, run `restoreSession()` with that layer (`Effect.provide(layer)`), then `ReactDOM.createRoot(...).render(...)`. No `SubscriptionRef`, no `createReactiveStoreFromRef`, no `setApplicationLayer`, no `Effect.never`. Depends on .2 and .3.
- **Claim:** `devenv shell -- bd update forge-z628.7 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.7 --reason "Done" --json`

### forge-z628.8 — [ReactiveStore] Remove createReactiveStoreFromRef
- **File:** `crates/forge-cli/templates/default/frontend/src/lib/ReactiveStore.ts`
- **Do:** Delete the function `createReactiveStoreFromRef` and its export. Remove any imports of it (e.g. in `main.tsx` already reverted in .7). Depends on .7.
- **Claim:** `devenv shell -- bd update forge-z628.8 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.8 --reason "Done" --json`

### forge-z628.9 — [Auth store] Ensure auth store uses makeReactiveStore layer only
- **File:** `crates/forge-cli/templates/default/frontend/src/features/authentication/stores/AuthenticationStateReactiveStore.ts`
- **Do:** Verify the module uses `makeReactiveStore(...)` and exports `getAuthenticationStateStoreLayer()` that returns that layer. No special wiring or override. If already correct, document that and close. Depends on .6.
- **Claim:** `devenv shell -- bd update forge-z628.9 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.9 --reason "Done" --json`

### forge-z628.10 — [Auth store] Export useAuthStore bound hook (useRunWithAppLayer + useReactiveStore)
- **Files:** `crates/forge-cli/templates/default/frontend/src/features/authentication/hooks/useAuthenticationStateReactiveStore.ts` and/or store index.
- **Do:** Export a bound hook so call sites do `useAuthStore()` or keep `useAuthenticationStateReactiveStore()` that returns `{ authentication, initialized }` and internally uses `useRunWithAppLayer()` + `useReactiveStoreWithInit(tag, initial, run, runFork)`. No tag/initial/run/runFork at call site. Depends on .9.
- **Claim:** `devenv shell -- bd update forge-z628.10 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.10 --reason "Done" --json`

### forge-z628.11 — [ReactiveStore] Add defineStore(name, initial) wrapper returning tag, layer, useStore
- **File:** `crates/forge-cli/templates/default/frontend/src/lib/ReactiveStore.ts`
- **Do:** Add `defineStore(name, initial)` that calls `makeReactiveStore(name, initial)` and returns `{ tag, layer, useStore, useStoreWithInit }` where `useStore` / `useStoreWithInit` are functions that return a hook (e.g. that call `useRunWithAppLayer` and `useReactiveStore`/`useReactiveStoreWithInit` with the stored tag and initial). If the library cannot import app layer, document that store modules define the bound hook themselves and return a placeholder or the same tag/layer only. Depends on .6.
- **Claim:** `devenv shell -- bd update forge-z628.11 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.11 --reason "Done" --json`

### forge-z628.12 — [Docs] Mark reactive-store-library-redesign plan as implemented
- **File:** `history/reactive-store-library-redesign.md`
- **Do:** Add an "Implemented" section at the top or bottom listing completed tickets (forge-z628.1–.11) and the date. Short note that the redesign is done. Depends on .7 and .8.
- **Claim:** `devenv shell -- bd update forge-z628.12 --status in_progress --claim --json`
- **Close:** `devenv shell -- bd close forge-z628.12 --reason "Done" --json`

---

## Quick reference: bd commands (all via `devenv shell --`)

- List open under epic: `bd list --parent forge-z628 -q`
- Ready (no unclosed deps): `bd ready --json`
- Claim: `bd update <id> --status in_progress --claim --json`
- Close: `bd close <id> --reason "Done" --json`
- Swarm status: `bd swarm status forge-z628`

---

## Parallel execution strategy

1. **First batch:** Claim and implement forge-z628.1, forge-z628.2, forge-z628.3 in parallel (three workers or three parallel invocations). Close each when done; commit code + `.beads/issues.jsonl`.
2. **Second batch:** After 1,2,3 are closed, claim and implement forge-z628.4 and forge-z628.5 in parallel.
3. **Third:** After 4,5 closed, claim and implement forge-z628.6.
4. **Fourth:** After 6 closed, claim and implement forge-z628.7, forge-z628.9, forge-z628.11 in parallel.
5. **Fifth:** After 7,9,11 closed, claim and implement forge-z628.8 and forge-z628.10 in parallel.
6. **Last:** After 8,10 closed, claim and implement forge-z628.12.

Use this brief as the single input; no need to look up bd or the plan beyond this file.
