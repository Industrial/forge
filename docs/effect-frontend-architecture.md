# Effect.ts Frontend Architecture

This document describes the architecture for the default Forge frontend: **Effect.ts as the implementation layer** and **react-effect** as the single boundary between Effect and React. It outlines where each piece of the codebase lives and the changes we will make to get there.

---

## Decisions (summary)

- **State:** Keep stateful code simple for now. No streams or subscription machinery. Use the existing react-effect helpers (e.g. `useEffectState`, `useActionStateEffect`) to run “get state” Effects and hold results in React state; re-run on mount and after mutations (login, logout, setScope).
- **Layers:** One layer per feature. Feature-specific Layers live in feature directories; global infrastructure (services used by multiple features) lives above features. At root we compose all Layers into one Runtime.
- **Code location:** Feature-specific code (effects, layer, pages, components) lives in feature directories. Global code (shared services, app bootstrap) lives above the feature directories.
- **Running Effects:** Prefer react-effect hooks; use `Runtime.runPromise(runtime)(effect)` from `useEffectRuntime()` only when the hooks don’t fit.
- **runEffect.runPromise:** Do not use for Effects with requirements (`R !== never`). Use the runtime from the provider (via hooks) so the app Layer is available.
- **HTTP:** Use Effect’s built-in HTTP client (`@effect/platform`, e.g. `HttpClient` / `FetchHttpClient`) for all backend communication. ApiClient wraps or uses this so we get consistent typing, error handling, and testability via Layers.
- **WebSocket:** Wrap the WebSocket in a **Service** (LiveWs). The service owns the connection and exposes incoming messages as an Effect **Stream** built from a **Queue** (`Stream.fromQueue(queue)`): the WS handler offers each message to the Queue; consumers get a `Stream<Message>` and use Stream operations in a Scope.

---

## 1. The boundary: react-effect

**react-effect** (`frontend/src/lib/react-effect`) is the **only** place where React and Effect meet.

- **EffectRuntimeProvider** wraps the app (or a subtree) and holds a `Runtime<R>`. That runtime is built once with the application’s **Layer** (e.g. ApiClient, AuthStore). All Effect runs triggered by the UI use this runtime, so they automatically have access to `R`.
- **Hooks** (`useRunEffect`, `useEffectState`, `useTransitionEffect`, `useActionStateEffect`, `useOptimisticEffect`, `useEffectReducer`) call `useEffectRuntime()` to get that runtime and run Effects with it. They never call `Effect.runPromise` or `Layer` directly; they submit an `Effect<A, E, R>` and the runtime executes it with the provided Layer.
- **React components** do not import `Effect`, `Layer`, or `Runtime` for app logic. They use the react-effect hooks and, when they need to run an Effect, they pass an Effect value (often from a function that returns `Effect<A, E, R>`). So the “divide” is: **Effect world owns types and programs; React world owns rendering and event handlers that invoke those programs via the hooks.**

Consequences:

- **Per-feature Layers, one Runtime**: Each feature defines its own Layer(s); at app root we merge or compose them and build a single Runtime. Tests replace the Layer (or the runtime) to inject mocks; no `vi.mock` of Effect or services.
- **Consistent cleanup**: `useRunEffect` runs Effects in a Scope and closes it on unmount/deps change, so subscriptions and finalizers behave like in pure Effect code.
- **Typed errors**: Effects carry `E`; hooks expose success/failure (e.g. `AsyncState`, or callback `onError`). UI can show errors without losing type information at the boundary if we pass through a small error type or a string derived from `E`.

---

## 2. Where things live

### 2.1 Effect land (no React)

**Location:** **Global** code (shared services, shared Layers) lives **above** the feature directories (e.g. `src/services/`, `src/schemas/`). **Feature-specific** code (programs, feature Layers) lives **inside** feature directories (e.g. `src/features/auth/`, `src/features/users/`). No React imports in Effect land.

| What | Where | Reason |
|------|--------|--------|
| **Global services (Tags)** | Above features, e.g. `src/services/ApiClient.ts`, `AuthStore.ts`, `LiveWs.ts` | Shared by multiple features; one Tag per file; no `R` in the interface (no requirement leakage). |
| **Global Live/Mock** | Above features, e.g. `src/services/ApiClientLive.ts`, `AuthStoreLive.ts`, `*Mock.ts` | `Layer.effect` or `Layer.succeed`; used by feature Layers or composed at root. |
| **Feature Layer** | Inside feature, e.g. `src/features/auth/layer.ts`, `src/features/users/layer.ts` | One layer per feature. Each feature’s Layer provides the services that feature needs; it may depend on global services. |
| **Runtime composition** | Above features, e.g. `src/app/runtime.ts` or root `main.tsx` / `App.tsx` | Merge or compose all feature Layers (and global Layers) once; build `Runtime.defaultRuntime.pipe(Runtime.provide(composedLayer))` and pass to `EffectRuntimeProvider`. |
| **Effect programs** | Inside feature, e.g. `src/features/auth/effects.ts`, `src/features/users/effects.ts` | Pure functions returning `Effect<A, E, R>`. Feature-specific programs live in that feature’s directory; they `yield*` services and do not use React. |
| **Schemas** | Global: `src/schemas/` or next to the effect that uses them | Effect Schema for request/response validation. Used inside Effect programs and optionally by `effectSchemaResolver`. |

**Principle:** Effect land is testable in isolation: run `Effect.runPromise(program.pipe(Effect.provide(mockLayer)))`. No DOM, no React.

---

### 2.2 React land (no Effect types in app logic)

**Location:** `src/components/`, `src/features/*/pages/`, `src/layouts/`, and any **React-only** hooks under `src/hooks/`.

| What | Where | Reason |
|------|--------|--------|
| **Components** | As today (components, features, layouts) | They render UI and call react-effect hooks. They receive Effect **values** (e.g. the result of `fetchUsersEffect()`) or callbacks that run Effects via hooks; they do not construct Layers or run `Effect.runPromise` themselves. |
| **UI-only state** | Local to components or small React state | Theme (light/dark), modal open/closed, form field values (if not driven by Effect). Purely presentational state stays in React. |
| **Route tree** | e.g. `App.tsx`, route config | Unchanged in spirit; we may wrap the tree with `EffectRuntimeProvider` at the top and remove legacy React Context that duplicated Effect-owned state (e.g. Auth). |
| **Guards / wrappers** | e.g. `ProtectedRoute`, `DashboardPermissionGuard` | Can stay React; they either read from a hook that exposes “current user / permissions” (which comes from Effect land via a hook) or trigger an Effect (e.g. “ensure session”) via react-effect. |

**Principle:** React code does not depend on `Effect` or `Layer` for **application** logic. It depends on **react-effect** and on functions that return Effects (which live in Effect land).

---

### 2.3 The boundary (react-effect and app bootstrap)

**Location:** `src/lib/react-effect/` (existing), plus **one place** above features that builds the runtime and wraps the app.

| What | Where | Reason |
|------|--------|--------|
| **EffectRuntimeProvider** | Used at root (e.g. `main.tsx` or `App.tsx`) | Wraps the app with a `Runtime<R>` built from the composed Layer (all feature Layers + global). All hooks under this tree get the same `R`. |
| **Runtime construction** | Above features, e.g. `src/app/runtime.ts` or next to root | Compose all feature Layers and global Layers; build `Runtime.defaultRuntime.pipe(Runtime.provide(composedLayer))`. Single place so every Effect run in the UI sees the same services. |
| **Hooks** | `src/lib/react-effect/*.ts` | Already implemented. They call `useEffectRuntime()` and run Effects with that runtime. No change to their API for this doc. |
| **effectSchemaResolver** | `src/lib/effectSchemaResolver.ts` | Stays in `lib/`: bridge from Effect Schema to React Hook Form. It does not run app Effects; it only validates form values. |

**Principle:** The only code that knows both “React” and “Runtime/Layer” is react-effect and the single bootstrap that creates the runtime and mounts the provider.

---

### 2.4 Auth and session: a concrete split

Today, auth lives in React Context (`context/Auth.tsx`): token, user, profiles, permissions, `fetchMe`, `logout`, etc.

**Target:**

- **Effect land:** An **AuthStore** service (Tag) with e.g. `getToken`, `setToken`, `getCurrentUser`, `getState`, `fetchMe`, `logout`, `setScope`. State (token, user, profiles, permissions) lives inside the Live implementation (e.g. in a Ref). Methods return `Effect<..., AuthError, never>` (no requirement leakage). `fetchMeEffect`, `loginEffect`, `logoutEffect` are Effect programs that use AuthStore (and optionally ApiClient). They are pure `Effect<A, E, R>`. Auth-specific Layer lives in the auth feature (e.g. `src/features/auth/layer.ts`).
- **React land:** No AuthProvider that holds token/user in React state. Instead:
  - A **useAuth()** hook that components use to read “current user, token, profiles, permissions”. Implement it in a **simple way**: run an Effect “get current auth state” (e.g. `getAuthStateEffect` or `AuthStore.getState()`) and store the result in React state using the existing react-effect helpers (e.g. `useEffectState`, or a small custom hook that runs the Effect and setState). Re-run that Effect on mount and after every login, logout, or setScope (e.g. by calling a “refresh” from the hook or by re-running when a dependency changes). **No streams or subscription machinery** for now.
  - For “run login” or “run logout”, components use existing hooks: e.g. `useActionStateEffect(loginEffect, idle())` or `useTransitionEffect()` and `startTransition(() => loginEffect)`.

So: **auth state and behavior live in Effect (AuthStore + programs); React only reads and triggers via react-effect hooks.** useAuth() uses the simple pattern: run getState Effect, store in state, re-run on mount and after mutations.

---

### 2.5 API client and data fetching

- **Effect land:** **ApiClient** service (Tag) with `get`, `post`, etc., returning `Effect<Response, ApiError, never>`. **ApiClientLive** is implemented using Effect’s HTTP client from `@effect/platform` (e.g. `HttpClient` / `FetchHttpClient`), and depends on AuthStore (or a minimal “token provider”) to add headers. All “fetch users”, “fetch organizations”, etc. are Effect programs that `yield* ApiClient` and return `Effect<A, E, ApiClient | ...>`.
- **React land:** Pages that need “list of users” use e.g. `useEffectState(null, onError)` and `setState(fetchUsersEffect)` on mount, or `useActionStateEffect(() => fetchUsersEffect(), idle())`, or a small custom hook that wraps one of these. So the **decision** “we need users” and “show loading/error” is in React; the **implementation** is in Effect.

**Remove:** Legacy `useApi()` that returns an imperative `api(url)` and any React Context that only existed to pass that down. The only “API” the UI sees is “run this Effect” (e.g. `fetchUsersEffect`), which already carries the dependency on ApiClient via `R`.

---

### 2.6 WebSocket and live updates

**Decision:** The WebSocket is wrapped in a **Service** (LiveWs). The service owns the connection and uses **Stream from Queue** for incoming messages.

- **LiveWs** (Tag) is a global service above features (e.g. `src/services/LiveWs.ts`). Its interface exposes at least a **stream of messages**, e.g. `messages: Stream.Stream<Message>` or a method that returns `Effect<Stream.Stream<Message>, LiveWsError, never>`. Optionally it also exposes connection state (e.g. `status: Ref<ConnectionStatus>`) or a way to send messages.
- **LiveWsLive** holds a **Queue** (bounded or unbounded). The WebSocket event handler (on message) offers each incoming message to the Queue. The service exposes that stream of messages as `Stream.fromQueue(queue)` (or equivalent) so consumers get an Effect `Stream<Message>`.
- Consumers (other services or Effect programs) depend on LiveWs and run Stream operations (`Stream.runForEach`, `Stream.take`, `Stream.map`, etc.) in a Scope. React code that needs live updates runs an Effect that consumes the stream (e.g. via `useRunEffect` or a custom hook that runs a program that subscribes to the stream and updates state), or a dedicated hook that subscribes to the stream and exposes the latest value(s) to the UI.
- **LiveWsMock** for tests provides a Queue (or a pre-fed stream) so tests can control which messages are delivered without a real WebSocket.

---

### 2.7 Where *not* to put things

- **Do not** put Effect programs (e.g. `fetchUsersEffect`) inside `context/` or inside components. Put them in Effect land (e.g. `effects/`, `services/`, or feature folders) and import them into components only to pass to hooks.
- **Do not** put Runtime construction inside components or feature UI code. Compose Layers at one place above features (e.g. `src/app/`); feature Layers are defined in feature directories.
- **Do not** use `runEffect.runPromise` (or raw `Effect.runPromise`) in components or in React hooks for app logic; they don’t see the app’s Runtime/Layer. Use the runtime from `useEffectRuntime()` or, better, the existing hooks that already use it.
- **Do not** duplicate “current user” or “token” in both React state and Effect; AuthStore (or equivalent) is the single source of truth, and React subscribes via a hook.

---

## 3. Changes we will make (summary)

1. **Introduce global services and per-feature Layers**  
   Add global services (e.g. ApiClient, AuthStore, optionally LiveWs) as Tags with Live/Mock above features (`src/services/`). Each feature that needs services defines its own Layer in its directory (e.g. `src/features/auth/layer.ts`, `src/features/users/layer.ts`). Compose all Layers in one place above features to build the runtime.

2. **Bootstrap runtime at root**  
   Compose global and feature Layers; build the runtime from that composition and wrap the app with `EffectRuntimeProvider` so all hooks use that runtime.

3. **Move auth into Effect**  
   Implement AuthStore (state in a Ref or similar), `fetchMeEffect`, `loginEffect`, `logoutEffect`, `setScopeEffect`. Provide **useAuth()** that runs a “get state” Effect and stores the result in React state using react-effect helpers; re-run on mount and after login/logout/setScope. No streams. Remove the old AuthProvider’s internal state and use AuthStore as source of truth.

4. **Move API usage into Effect**  
   Implement ApiClient and ApiClientLive using Effect’s HTTP client (`@effect/platform`); rewrite `fetchUsersEffect` and all other API calls as Effects depending on ApiClient. Replace `useApi()` usage in the UI with “run this Effect” via react-effect hooks. Remove `useApi()` and any context that only passed it.

5. **WebSocket / live updates**  
   Implement LiveWs as a Service that owns the WebSocket connection and exposes a message stream via **Stream.fromQueue**: LiveWsLive uses a Queue, the WS handler offers each message to the Queue, and the service exposes `Stream.fromQueue(queue)` (or equivalent) so consumers get an Effect `Stream<Message>`. Add LiveWsMock for tests.

6. **Use react-effect everywhere for effectful UI**  
   For any “load data”, “submit form”, “login”, “logout”, use one of: `useRunEffect`, `useEffectState`, `useTransitionEffect`, `useActionStateEffect`, `useOptimisticEffect`, `useEffectReducer`. Prefer these hooks; avoid ad hoc `Effect.runPromise` in components.

7. **Keep effectSchemaResolver**  
   It stays the bridge from Effect Schema to React Hook Form; no change to the Layer/runtime story.

8. **Tests**  
   Effect programs are tested with `Effect.runPromise(program.pipe(Effect.provide(mockLayer)))`. Component tests wrap the tree in `EffectRuntimeProvider` with a test Layer. No `vi.mock` of Effect or of our services.

---

## 4. Rationale (placement and tradeoffs)

### 4.1 Auth state in Effect, useAuth() via helpers

Auth state lives in Effect (AuthStore) so it’s the single source of truth and ApiClient (and other services) can depend on it. useAuth() runs a “get state” Effect and stores the result in React state using react-effect helpers; we re-run on mount and after login/logout/setScope. We keep this simple: no streams or subscription machinery.

### 4.2 One layer per feature, one Runtime at root

Each feature defines its own Layer(s) in its directory. Global services (e.g. ApiClient, AuthStore) live above features with their Layers. At root we compose all Layers and build a single Runtime. So: one Runtime for the app, but Layer definitions are owned per feature (and global where shared).

### 4.3 Feature code in feature dirs, global above

Feature-specific code (Effect programs, feature Layer, pages, components) lives in feature directories. Global code (shared services, Layer composition, runtime construction) lives above the feature directories. This keeps features self-contained and shared infrastructure in one place.

### 4.4 Prefer hooks; direct runPromise as exception

Components should use react-effect hooks to run Effects. Use `Runtime.runPromise(runtime)(effect)` from `useEffectRuntime()` only when the hooks don’t fit (e.g. a non-React callback).

### 4.5 runEffect.runPromise only for R = never

Do not use `runEffect.runPromise` for any Effect that has requirements; it uses the default runtime and has no app Layer. Use the runtime from the provider (via the hooks) for app code.

---

## 5. Summary

| Layer | Contents | Rule |
|-------|----------|------|
| **Effect land** | Global services above features; per-feature Layers and Effect programs in feature dirs | No React. Test with `Effect.provide(mockLayer)`. |
| **Boundary** | react-effect (provider + hooks), runtime composition (above features), effectSchemaResolver | Only place that knows both React and Runtime/Layer. |
| **React land** | Components, routes, UI state, useAuth() and similar “read from Effect” hooks | Prefer hooks; no direct Effect.runPromise or Layer for app logic. |

**react-effect** is the divide: Effect.ts owns implementation and state (including auth); React owns rendering and user events; the hooks are the only bridge. State is kept simple (no streams); one layer per feature; feature code in feature dirs, global code above.
