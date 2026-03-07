# Reactive Store Redesign — Files to Change

## 1. Store definition (DSL)

| File | Change |
|------|--------|
| `features/authentication/stores/AuthenticationStateReactiveStore.ts` | Export `AuthStore` = defineStore result; `AuthStoreTag`, `authStoreLayer`; keep `getAuthenticationStateStoreLayer()`, types, and backward-compat tag alias. |
| `features/authentication/stores/index.ts` | Re-export `useAuthStore`, `useAuthStoreWithInit`; export `AuthStoreTag`, `authStoreLayer`. |

## 2. App layer

| File | Change |
|------|--------|
| `lib/appLayer.ts` | No change (already uses `getAuthenticationStateStoreLayer()`). |

## 3. Main entry

| File | Change |
|------|--------|
| `main.tsx` | No change (already simple: get layer, restoreSession, render). |

## 4. Consumers (use bound hooks, import from stores)

| File | Change |
|------|--------|
| `components/ProtectedRoute.tsx` | `useAuthStoreWithInit()` from `@/features/authentication/stores`. |
| `components/Navbar.tsx` | `useAuthStore()` from stores. |
| `components/GuestRoute.tsx` | `useAuthStore()` from stores. |
| `components/SubscriptionStreamRunner.tsx` | `useAuthStore()` from stores. |
| `components/DashboardPermissionGuard.tsx` | `useAuthStore()` from stores. |
| `components/DashboardScopeGuard.tsx` | `useAuthStore()` from stores. |
| `features/authentication/components/SelectScopeOnlyGuard.tsx` | `useAuthStore()` from stores. |
| `features/authentication/pages/SelectScopePage/SelectScopePage.tsx` | `useAuthStore()` from stores. |
| `features/dashboard/components/Sidebar/Sidebar.tsx` | `useAuthStore()` from stores. |
| `features/profile/pages/ProfilePage/ProfilePage.tsx` | `useAuthStore()` from stores. |
| `hooks/usePermission.ts` | `useAuthStore()` from stores. |
| `hooks/useEntitySubscription.ts` | `useAuthStore()` from stores. |

## 5. Library

| File | Change |
|------|--------|
| `lib/ReactiveStore.ts` | No change (defineStore + Layer.sync already in place). |

## 6. Optional

| File | Change |
|------|--------|
| `lib/subscriptionStreamStatusStore.ts` | Align to same DSL pattern (export store object, tag, layer). |
