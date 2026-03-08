# Automatic Query-Based Subscriptions: `useEntityQuery` Implementation Plan

## Executive Summary

This document describes the implementation plan for automatic query-based subscriptions, enabling developers to write queries without manually managing subscriptions. When a developer queries data, the system will automatically subscribe to changes, handle invalidations, and refetch data seamlessly.

**Goal**: Developers write `useEntityQuery("blogPost", params)` and get live, automatically-updated data without any subscription management code.

---

## 1. Current State Analysis

### 1.1 Current Architecture

**Backend (Rust)**:
- ✅ RPC endpoint `/api/rpc` with `subscribe` method accepts query params (filter, sort, pagination)
- ✅ Subscription store (`SubscriptionStore`) tracks subscriptions by ID
- ✅ SSE stream endpoint `/api/subscriptions/stream` delivers invalidation events
- ✅ Change events published after CUD operations trigger matching subscriptions
- ✅ Subscription matching uses entity_id + scope + query params

**Frontend (TypeScript/React)**:
- ✅ `EntityApi` service for REST queries (`list`, `get`, `create`, `update`, `delete`)
- ✅ `RpcApi` service for subscription management (`subscribe`)
- ✅ `SubscriptionStream` service for SSE stream connection
- ✅ `useEntitySubscription` hook for manual subscription management
- ✅ `subscriptionRegistry` for tracking subscriptions and triggering refetches
- ✅ `SubscriptionStreamRunner` component manages global SSE connection

### 1.2 Current Developer Experience

**Current Pattern** (Manual):
```typescript
// Developer must:
// 1. Query data
const { data, refetch } = useEntityList("blogPost", params)

// 2. Manually subscribe
useEntitySubscription("blogPost", params, () => {
  refetch()
})

// 3. Handle cleanup manually
```

**Problems**:
- ❌ Two separate hooks required
- ❌ Developer must wire up refetch callback
- ❌ Easy to forget subscription
- ❌ No automatic cleanup coordination
- ❌ No query deduplication
- ❌ No support for nested/related queries

### 1.3 Existing Infrastructure

**What Works**:
- Backend subscription system is complete and functional
- Frontend has all necessary services (`EntityApi`, `RpcApi`, `SubscriptionStream`)
- Subscription registry tracks subscriptions correctly
- SSE stream delivers invalidations reliably

**What's Missing**:
- Unified hook that combines querying + subscribing
- Automatic subscription lifecycle management
- Query deduplication (multiple components querying same data)
- Support for nested queries (blog posts + comments)
- Automatic cleanup coordination

---

## 2. Desired State

### 2.1 Developer Experience

**Target Pattern** (Automatic):
```typescript
// Developer just writes:
const { data, loading, error } = useEntityQuery("blogPost", {
  filter: { published: true },
  sort: { field: "date", order: "desc" },
  limit: 10
})

// Automatically:
// ✅ Queries data on mount
// ✅ Subscribes to changes
// ✅ Refetches on invalidation
// ✅ Unsubscribes on unmount
// ✅ Handles loading/error states
```

**Nested Queries**:
```typescript
// Blog posts with comments
const { data: posts } = useEntityQuery("blogPost", {
  filter: { published: true },
  sort: { field: "date", order: "desc" },
  limit: 10
})

// Automatically detect and subscribe to related entities
const { data: comments } = useEntityQuery("blogPostComment", {
  filter: { 
    blog_post_id: { in: posts.map(p => p.id) }
  },
  sort: { field: "upvotes", order: "desc" },
  limit: 3
})
```

### 2.2 Requirements

1. **Automatic Subscription**: Query automatically subscribes with same params
2. **Automatic Refetch**: Invalidation events trigger automatic refetch
3. **Automatic Cleanup**: Unsubscribe when component unmounts or params change
4. **Query Deduplication**: Multiple components querying same data share one subscription
5. **Nested Query Support**: Detect and subscribe to related entities
6. **Loading States**: Handle pending/success/error states
7. **Type Safety**: Full TypeScript support with proper types
8. **Backward Compatible**: Existing `useEntitySubscription` continues to work

---

## 3. Architecture Design

### 3.1 Component Hierarchy

```
useEntityQuery (public API)
  ├── useEntityQueryInternal (core logic)
  │   ├── useEntityQueryState (state management)
  │   ├── useEntitySubscriptionManager (subscription lifecycle)
  │   └── useQueryDeduplication (shared subscriptions)
  └── useNestedQueries (related entity detection)
```

### 3.2 Data Flow

```
Component mounts
  ↓
useEntityQuery called
  ↓
1. Check if query already subscribed (deduplication)
  ├── Yes: Reuse subscription, refetch data
  └── No: Create new subscription
  ↓
2. Subscribe via RpcApi.subscribe(entityId, params)
  ↓
3. Store subscription_id in registry
  ↓
4. Query data via EntityApi.list(entityId, params)
  ↓
5. Return data + loading state
  ↓
[Component renders]
  ↓
Change event occurs (backend)
  ↓
SSE stream receives invalidation event
  ↓
SubscriptionStreamRunner triggers registry
  ↓
Registry calls onInvalidate callback
  ↓
useEntityQuery refetches data
  ↓
Component re-renders with new data
  ↓
Component unmounts
  ↓
Unsubscribe (if no other components using same query)
```

### 3.3 Subscription Lifecycle

**Subscription States**:
- `idle`: No subscription yet
- `subscribing`: RPC subscribe call in progress
- `subscribed`: Subscription active, waiting for data
- `ready`: Data loaded, subscription active
- `unsubscribing`: Cleanup in progress
- `unsubscribed`: Cleaned up

**Lifecycle Management**:
- **Mount**: Subscribe → Query → Ready
- **Params Change**: Unsubscribe old → Subscribe new → Query → Ready
- **Unmount**: Unsubscribe → Cleanup
- **Error**: Retry subscription or fail gracefully

### 3.4 Query Deduplication Strategy

**Problem**: Multiple components query same entity + params

**Solution**: Subscription manager tracks active subscriptions by query key

**Query Key Format**:
```typescript
type QueryKey = `${entityId}:${JSON.stringify(normalizedParams)}`
```

**Deduplication Logic**:
```typescript
const subscriptionRefs = new Map<QueryKey, {
  subscriptionId: string
  refCount: number
  callbacks: Set<() => void>
}>()

// On subscribe:
if (subscriptionRefs.has(queryKey)) {
  // Increment ref count, add callback
  subscriptionRefs.get(queryKey).refCount++
  subscriptionRefs.get(queryKey).callbacks.add(onInvalidate)
} else {
  // Create new subscription
  const subscriptionId = await rpc.subscribe(entityId, params)
  subscriptionRefs.set(queryKey, {
    subscriptionId,
    refCount: 1,
    callbacks: new Set([onInvalidate])
  })
}

// On unsubscribe:
const ref = subscriptionRefs.get(queryKey)
ref.refCount--
ref.callbacks.delete(onInvalidate)
if (ref.refCount === 0) {
  // Actually unsubscribe from server
  await rpc.unsubscribe(ref.subscriptionId)
  subscriptionRefs.delete(queryKey)
}
```

### 3.5 Nested Query Detection

**Problem**: Blog post page queries posts, then queries comments for those posts

**Solution**: Detect related entity queries and subscribe automatically

**Detection Strategy**:
1. Analyze query params for foreign key filters
2. Track parent query results
3. When parent data changes, detect if nested query needs update
4. Subscribe to nested entity changes

**Example**:
```typescript
// Parent query
const { data: posts } = useEntityQuery("blogPost", params)

// Nested query (detected automatically)
const { data: comments } = useEntityQuery("blogPostComment", {
  filter: { blog_post_id: { in: posts.map(p => p.id) } }
})

// System automatically:
// 1. Detects blog_post_id filter
// 2. Subscribes to blogPostComment changes
// 3. Also subscribes to blogPost changes (parent)
// 4. When parent changes, updates nested query params
```

**Implementation Notes**:
- Use query param analysis to detect foreign keys
- Track parent-child relationships
- Update nested query when parent data changes
- Handle dynamic filter updates (e.g., `in: [ids]`)

---

## 4. Implementation Details

### 4.1 Core Hook: `useEntityQuery`

**Location**: `frontend/src/hooks/useEntityQuery.ts`

**Signature**:
```typescript
interface UseEntityQueryOptions {
  enabled?: boolean  // Allow disabling query/subscription
  refetchOnMount?: boolean  // Refetch when component mounts
  refetchInterval?: number  // Polling interval (optional)
  staleTime?: number  // Consider data fresh for N ms
}

function useEntityQuery<T = unknown>(
  entityId: string,
  params?: ListQueryParams,
  options?: UseEntityQueryOptions
): {
  data: T[] | undefined
  loading: boolean
  error: Error | null
  refetch: () => Promise<void>
  subscriptionId: string | null
  isSubscribed: boolean
}
```

**Implementation Steps**:

1. **State Management**:
   ```typescript
   const [state, setState] = useState<{
     data: T[] | undefined
     loading: boolean
     error: Error | null
     subscriptionId: string | null
   }>({
     data: undefined,
     loading: true,
     error: null,
     subscriptionId: null
   })
   ```

2. **Query Key Generation**:
   ```typescript
   const queryKey = useMemo(
     () => generateQueryKey(entityId, params),
     [entityId, JSON.stringify(normalizedParams)]
   )
   ```

3. **Subscription Management**:
   ```typescript
   useEffect(() => {
     if (!enabled) return
     
     let cancelled = false
     
     const setupSubscription = async () => {
       // 1. Check deduplication
       const existing = subscriptionManager.get(queryKey)
       if (existing) {
         existing.addCallback(handleInvalidate)
         setState(prev => ({ ...prev, subscriptionId: existing.id }))
         // Refetch if needed
         if (refetchOnMount) {
           await refetchData()
         }
         return
       }
       
       // 2. Create subscription
       setState(prev => ({ ...prev, loading: true }))
       try {
         const result = await rpc.subscribe(entityId, params)
         if (cancelled) return
         
         const subscriptionId = result.subscription_id
         
         // 3. Register in deduplication manager
         subscriptionManager.register(queryKey, {
           id: subscriptionId,
           addCallback: (cb) => callbacks.add(cb),
           removeCallback: (cb) => callbacks.delete(cb)
         })
         
         // 4. Register in subscription registry
         register(subscriptionId, {
           entityId,
           params,
           onInvalidate: handleInvalidate
         })
         
         // 5. Initial data fetch
         await refetchData()
         
         setState(prev => ({
           ...prev,
           subscriptionId,
           loading: false
         }))
       } catch (error) {
         if (cancelled) return
         setState(prev => ({
           ...prev,
           error: error as Error,
           loading: false
         }))
       }
     }
     
     setupSubscription()
     
     return () => {
       cancelled = true
       const existing = subscriptionManager.get(queryKey)
       if (existing) {
         existing.removeCallback(handleInvalidate)
         if (existing.refCount === 0) {
           // Actually unsubscribe
           rpc.unsubscribe(existing.id)
           subscriptionManager.unregister(queryKey)
         }
       }
     }
   }, [queryKey, enabled, refetchOnMount])
   ```

4. **Data Fetching**:
   ```typescript
   const refetchData = useCallback(async () => {
     try {
       setState(prev => ({ ...prev, loading: true, error: null }))
       const result = await entityApi.list(entityId, params)
       setState(prev => ({
         ...prev,
         data: result.data as T[],
         loading: false
       }))
     } catch (error) {
       setState(prev => ({
         ...prev,
         error: error as Error,
         loading: false
       }))
     }
   }, [entityId, params])
   ```

5. **Invalidation Handler**:
   ```typescript
   const handleInvalidate = useCallback(() => {
     refetchData()
   }, [refetchData])
   ```

### 4.2 Subscription Manager: Query Deduplication

**Location**: `frontend/src/lib/subscriptionManager.ts`

**Purpose**: Track active subscriptions and share them across components

**Implementation**:
```typescript
interface SubscriptionRef {
  subscriptionId: string
  refCount: number
  callbacks: Set<() => void>
  entityId: string
  params?: ListQueryParams
}

class SubscriptionManager {
  private subscriptions = new Map<string, SubscriptionRef>()
  
  get(queryKey: string): SubscriptionRef | undefined {
    return this.subscriptions.get(queryKey)
  }
  
  register(
    queryKey: string,
    subscriptionId: string,
    entityId: string,
    params?: ListQueryParams
  ): SubscriptionRef {
    const existing = this.subscriptions.get(queryKey)
    if (existing) {
      existing.refCount++
      return existing
    }
    
    const ref: SubscriptionRef = {
      subscriptionId,
      refCount: 1,
      callbacks: new Set(),
      entityId,
      params
    }
    this.subscriptions.set(queryKey, ref)
    return ref
  }
  
  unregister(queryKey: string): boolean {
    const ref = this.subscriptions.get(queryKey)
    if (!ref) return false
    
    ref.refCount--
    if (ref.refCount === 0) {
      this.subscriptions.delete(queryKey)
      return true  // Actually unsubscribe needed
    }
    return false  // Still in use
  }
  
  addCallback(queryKey: string, callback: () => void): void {
    const ref = this.subscriptions.get(queryKey)
    if (ref) {
      ref.callbacks.add(callback)
    }
  }
  
  removeCallback(queryKey: string, callback: () => void): void {
    const ref = this.subscriptions.get(queryKey)
    if (ref) {
      ref.callbacks.delete(callback)
    }
  }
  
  trigger(queryKey: string): void {
    const ref = this.subscriptions.get(queryKey)
    if (ref) {
      ref.callbacks.forEach(cb => cb())
    }
  }
}

export const subscriptionManager = new SubscriptionManager()
```

**Integration with Subscription Registry**:
- When subscription created: Register in both `subscriptionManager` and `subscriptionRegistry`
- When invalidation received: `subscriptionRegistry` triggers → find query key → `subscriptionManager.trigger()`
- When cleanup: Check `subscriptionManager.refCount` → if 0, call `rpc.unsubscribe()`

### 4.3 Query Key Generation

**Location**: `frontend/src/lib/queryKey.ts`

**Purpose**: Generate stable, comparable keys for query deduplication

**Implementation**:
```typescript
function normalizeParams(params?: ListQueryParams): ListQueryParams {
  if (!params) return {}
  
  return {
    filter: params.filter ? normalizeFilter(params.filter) : undefined,
    sort: params.sort,
    order: params.order,
    offset: params.offset ?? 0,
    limit: params.limit
  }
}

function normalizeFilter(filter: unknown): unknown {
  // Normalize filter to canonical form
  // - Sort filter conditions consistently
  // - Normalize value types (string vs number)
  // - Handle nested filters
  return filter  // Simplified
}

export function generateQueryKey(
  entityId: string,
  params?: ListQueryParams
): string {
  const normalized = normalizeParams(params)
  return `${entityId}:${JSON.stringify(normalized)}`
}
```

**Requirements**:
- Must be deterministic (same params → same key)
- Must handle undefined/null values consistently
- Must normalize filter order (if order doesn't matter)
- Must handle nested objects/arrays

### 4.4 Nested Query Support

**Location**: `frontend/src/hooks/useNestedQueries.ts`

**Purpose**: Detect and manage related entity queries

**Detection Strategy**:
1. Analyze query params for foreign key patterns
2. Track parent query dependencies
3. Update nested queries when parent changes

**Implementation**:
```typescript
interface NestedQueryConfig {
  parentEntityId: string
  foreignKeyField: string
  childEntityId: string
}

function detectNestedQueries(
  queries: Map<string, { entityId: string; params?: ListQueryParams }>
): NestedQueryConfig[] {
  const nested: NestedQueryConfig[] = []
  
  // Analyze all queries for foreign key relationships
  queries.forEach((query, key) => {
    if (query.params?.filter) {
      // Check for foreign key filters like { blog_post_id: { in: [...] } }
      const fkFields = detectForeignKeyFilters(query.params.filter)
      fkFields.forEach(fk => {
        // Find parent query
        const parentKey = findParentQuery(fk.entityId, queries)
        if (parentKey) {
          nested.push({
            parentEntityId: fk.entityId,
            foreignKeyField: fk.field,
            childEntityId: query.entityId
          })
        }
      })
    }
  })
  
  return nested
}

function useNestedQueryUpdates(
  parentQueryKey: string,
  parentData: unknown[],
  nestedConfig: NestedQueryConfig
): void {
  // When parent data changes, update nested query params
  useEffect(() => {
    if (!parentData.length) return
    
    const ids = parentData.map(item => item[nestedConfig.foreignKeyField])
    
    // Update nested query with new IDs
    // This triggers useEntityQuery to update params
  }, [parentData, nestedConfig])
}
```

**Integration**:
- `useEntityQuery` analyzes its params for foreign key patterns
- When detected, registers as nested query
- When parent data changes, updates nested query params
- Nested query automatically re-subscribes with new params

### 4.5 RPC Unsubscribe Support

**Current State**: Backend has `unsubscribe` RPC method, but frontend may not use it

**Required**: Ensure `RpcApi` service has `unsubscribe` method

**Location**: `frontend/src/services/RpcApi.ts`

**Update**:
```typescript
export interface RpcApiService {
  subscribe: (entityId: string, params?: ListQueryParams) => Effect.Effect<SubscribeResult, Error, never>
  unsubscribe: (subscriptionId: string) => Effect.Effect<void, Error, never>  // Add this
}
```

**Implementation** (`RpcApiLive.ts`):
```typescript
const unsubscribe: RpcApiService['unsubscribe'] = (subscriptionId) =>
  Effect.gen(function* () {
    const client = yield* HttpClient.HttpClient
    const authStore = yield* AuthenticationStateReactiveStoreTag
    
    const state = yield* authStore.get()
    const token = Option.getOrElse(state.token, () => null)
    if (!token) {
      return yield* Effect.fail(new Error('Not authenticated'))
    }
    
    const url = `${baseUrl}/api/rpc`
    const body = {
      method: 'unsubscribe',
      subscription_id: subscriptionId
    }
    
    const req = HttpClientRequest.post(url)
      .pipe(
        HttpClientRequest.setHeader('Authorization', `Bearer ${token}`),
        HttpClientRequest.setHeader('Content-Type', 'application/json'),
        HttpClientRequest.setBody(JSON.stringify(body))
      )
    
    const response = yield* client.execute(req).pipe(Effect.mapError(toError))
    
    if (!response.status.toString().startsWith('2')) {
      return yield* Effect.fail(new Error(`Unsubscribe failed: ${response.status}`))
    }
  })
```

---

## 5. Edge Cases and Considerations

### 5.1 Authentication State Changes

**Problem**: User logs out while subscriptions active

**Solution**:
- `useEntityQuery` checks auth state
- If token becomes invalid, cancel all subscriptions
- Clear subscription manager and registry
- Reset query state

**Implementation**:
```typescript
const authState = useAuthStore()
const hasToken = Option.isSome(authState.token)

useEffect(() => {
  if (!hasToken) {
    // Cancel all subscriptions for this component
    // Clear state
    setState({
      data: undefined,
      loading: false,
      error: null,
      subscriptionId: null
    })
  }
}, [hasToken])
```

### 5.2 Network Errors

**Problem**: Subscription or query fails due to network error

**Solution**:
- Retry logic with exponential backoff
- Show error state to user
- Allow manual retry via `refetch()`

**Implementation**:
```typescript
const [retryCount, setRetryCount] = useState(0)
const MAX_RETRIES = 3

const subscribeWithRetry = async () => {
  for (let i = 0; i < MAX_RETRIES; i++) {
    try {
      return await rpc.subscribe(entityId, params)
    } catch (error) {
      if (i === MAX_RETRIES - 1) throw error
      await delay(Math.pow(2, i) * 1000)  // Exponential backoff
    }
  }
}
```

### 5.3 Rapid Param Changes

**Problem**: User changes filter rapidly, causing subscription churn

**Solution**:
- Debounce param changes
- Cancel in-flight subscriptions
- Only subscribe to latest params

**Implementation**:
```typescript
const debouncedParams = useDebounce(params, 300)

useEffect(() => {
  // Only subscribe when debounced params stabilize
}, [debouncedParams])
```

### 5.4 Large Result Sets

**Problem**: Query returns thousands of items, subscription invalidates frequently

**Solution**:
- Server-side pagination (already supported)
- Client-side virtualization for rendering
- Subscription still works, but only refetch visible page

**Note**: This is more of a UI concern, but `useEntityQuery` should handle pagination correctly.

### 5.5 Subscription Stream Disconnection

**Problem**: SSE stream disconnects, invalidations stop arriving

**Solution**:
- `SubscriptionStreamRunner` already handles reconnection
- `useEntityQuery` can check `isSubscribed` status
- Show connection status to user (optional)

**Implementation**:
```typescript
const streamStatus = useReactiveStore(
  SubscriptionStreamStatusStoreTag,
  initialSubscriptionStreamStatus
)

// If stream disconnected but subscription exists, show warning
if (subscriptionId && !streamStatus.connected) {
  // Stream disconnected, but subscription still registered
  // SubscriptionStreamRunner will reconnect automatically
}
```

### 5.6 Memory Leaks

**Problem**: Subscriptions not cleaned up properly

**Solution**:
- Strict cleanup in `useEffect` return function
- Track all subscriptions in subscription manager
- Cleanup on unmount guaranteed
- Test with React StrictMode (double mount/unmount)

### 5.7 Type Safety

**Problem**: `useEntityQuery` returns `unknown[]` by default

**Solution**:
- Generic type parameter: `useEntityQuery<BlogPost>(...)`
- Type inference from entity registry (future enhancement)
- Type-safe params based on entity schema

**Implementation**:
```typescript
function useEntityQuery<T = unknown>(
  entityId: string,
  params?: ListQueryParams,
  options?: UseEntityQueryOptions
): {
  data: T[] | undefined
  // ...
}
```

---

## 6. Migration Strategy

### 6.1 Backward Compatibility

**Requirement**: Existing code using `useEntitySubscription` continues to work

**Strategy**:
- Keep `useEntitySubscription` unchanged
- `useEntityQuery` uses same underlying services
- Both can coexist
- Gradual migration path

### 6.2 Migration Steps

1. **Phase 1: Core Implementation**
   - Implement `useEntityQuery` hook
   - Implement subscription manager
   - Add tests
   - Document API

2. **Phase 2: Integration**
   - Update `GenericEntityCrud` to use `useEntityQuery`
   - Update example pages
   - Add nested query support (optional)

3. **Phase 3: Migration**
   - Migrate existing pages to `useEntityQuery`
   - Remove `useEntitySubscription` usage (or keep for advanced cases)
   - Update documentation

4. **Phase 4: Optimization**
   - Add query deduplication
   - Add nested query detection
   - Performance tuning

### 6.3 Breaking Changes

**None**: `useEntityQuery` is additive, doesn't break existing code

**Deprecation** (Future):
- After migration complete, consider deprecating `useEntitySubscription`
- Keep for advanced use cases (manual control)

---

## 7. Testing Strategy

### 7.1 Unit Tests

**File**: `frontend/src/hooks/useEntityQuery.test.ts`

**Test Cases**:
1. ✅ Subscribes on mount
2. ✅ Queries data on mount
3. ✅ Unsubscribes on unmount
4. ✅ Refetches on invalidation
5. ✅ Handles subscription errors
6. ✅ Handles query errors
7. ✅ Updates when params change
8. ✅ Deduplicates multiple components
9. ✅ Cleans up on auth state change
10. ✅ Handles rapid param changes

### 7.2 Integration Tests

**File**: `frontend/src/hooks/useEntityQuery.integration.test.ts`

**Test Cases**:
1. ✅ Full flow: subscribe → query → invalidate → refetch
2. ✅ Multiple components sharing subscription
3. ✅ Nested queries (if implemented)
4. ✅ Network error recovery
5. ✅ Stream disconnection recovery

### 7.3 E2E Tests

**File**: `test/e2e/tests/015_live_queries.test.ts`

**Test Cases**:
1. ✅ Page loads, data appears
2. ✅ Create entity, data updates automatically
3. ✅ Update entity, data updates automatically
4. ✅ Delete entity, data updates automatically
5. ✅ Multiple tabs, changes sync
6. ✅ Filter change, subscription updates

---

## 8. Performance Considerations

### 8.1 Subscription Overhead

**Concern**: Too many subscriptions

**Mitigation**:
- Query deduplication reduces actual subscriptions
- Server-side subscription matching is efficient
- SSE stream is lightweight

### 8.2 Refetch Frequency

**Concern**: Too many refetches on invalidation

**Mitigation**:
- Debounce invalidations (optional)
- Batch refetches (optional)
- Only refetch visible queries (future optimization)

### 8.3 Memory Usage

**Concern**: Subscription manager memory

**Mitigation**:
- Cleanup on unmount
- Limit subscription history (optional)
- Garbage collect unused subscriptions

---

## 9. Documentation Requirements

### 9.1 API Documentation

**Location**: `frontend/src/hooks/useEntityQuery.ts` (JSDoc)

**Required**:
- Hook signature and parameters
- Return value description
- Usage examples
- Edge cases and warnings

### 9.2 Developer Guide

**Location**: `docs/frontend/useEntityQuery.md`

**Required**:
- Getting started guide
- Common patterns
- Advanced usage (nested queries, manual control)
- Migration guide from `useEntitySubscription`
- Troubleshooting

### 9.3 Examples

**Location**: `frontend/src/examples/useEntityQuery/`

**Required**:
- Basic usage
- With filters and sorting
- Nested queries
- Error handling
- Custom options

---

## 10. Implementation Checklist

### Phase 1: Core Infrastructure
- [ ] Implement `subscriptionManager.ts` (query deduplication)
- [ ] Implement `queryKey.ts` (key generation)
- [ ] Update `RpcApi` service with `unsubscribe` method
- [ ] Update `RpcApiLive` with unsubscribe implementation
- [ ] Update `subscriptionRegistry` to work with subscription manager

### Phase 2: Core Hook
- [ ] Implement `useEntityQuery.ts` hook
- [ ] Add state management (loading, error, data)
- [ ] Add subscription lifecycle management
- [ ] Add query deduplication integration
- [ ] Add cleanup logic
- [ ] Add error handling and retries

### Phase 3: Integration
- [ ] Update `GenericEntityCrud` to use `useEntityQuery`
- [ ] Update example pages
- [ ] Add TypeScript types
- [ ] Add JSDoc documentation

### Phase 4: Testing
- [ ] Write unit tests for `useEntityQuery`
- [ ] Write unit tests for `subscriptionManager`
- [ ] Write integration tests
- [ ] Write E2E tests
- [ ] Test edge cases (auth changes, network errors, etc.)

### Phase 5: Advanced Features (Optional)
- [ ] Implement nested query detection
- [ ] Add query result caching
- [ ] Add stale-while-revalidate pattern
- [ ] Add polling support (`refetchInterval`)

### Phase 6: Documentation
- [ ] Write API documentation
- [ ] Write developer guide
- [ ] Create usage examples
- [ ] Update FUNCTIONALITY_AND_TESTS.md

---

## 11. Future Enhancements

### 11.1 Query Result Caching

**Idea**: Cache query results across components

**Benefit**: Instant data for repeated queries

**Implementation**: Add cache layer between `useEntityQuery` and `EntityApi`

### 11.2 Optimistic Updates

**Idea**: Update UI immediately, then sync with server

**Benefit**: Perceived performance improvement

**Implementation**: Add optimistic update support to `useEntityQuery`

### 11.3 Query Prefetching

**Idea**: Prefetch queries likely to be needed

**Benefit**: Faster page transitions

**Implementation**: Analyze navigation patterns, prefetch on hover/focus

### 11.4 Subscription Analytics

**Idea**: Track subscription usage and performance

**Benefit**: Identify optimization opportunities

**Implementation**: Add telemetry to subscription manager

---

## 12. Open Questions

1. **Nested Queries**: Should we implement automatic nested query detection in Phase 1, or defer to Phase 5?

2. **Query Caching**: Should `useEntityQuery` cache results, or rely on React state only?

3. **Stale Data**: Should we show stale data while refetching, or show loading state?

4. **Error Recovery**: Should failed subscriptions auto-retry, or require manual retry?

5. **Subscription Limits**: Should we limit number of concurrent subscriptions per user?

6. **Type Inference**: Can we infer entity types from entity registry, or require manual typing?

---

## 13. Success Criteria

### 13.1 Functional Requirements
- ✅ Developer can query data with single hook call
- ✅ Data automatically updates on changes
- ✅ No manual subscription management required
- ✅ Works with existing backend infrastructure
- ✅ Backward compatible with existing code

### 13.2 Performance Requirements
- ✅ Subscription overhead < 10ms per query
- ✅ Refetch latency < 100ms (network dependent)
- ✅ Memory usage reasonable (< 1MB for 100 subscriptions)
- ✅ No memory leaks

### 13.3 Developer Experience
- ✅ API is intuitive and easy to use
- ✅ TypeScript types are accurate
- ✅ Error messages are helpful
- ✅ Documentation is complete
- ✅ Examples are clear

---

## Appendix A: Example Usage

### Basic Query
```typescript
function BlogPostList() {
  const { data, loading, error } = useEntityQuery<BlogPost>("blogPost", {
    filter: { published: true },
    sort: { field: "date", order: "desc" },
    limit: 10
  })
  
  if (loading) return <Loading />
  if (error) return <Error message={error.message} />
  
  return (
    <div>
      {data?.map(post => <BlogPostCard key={post.id} post={post} />)}
    </div>
  )
}
```

### With Filters
```typescript
function FilteredPosts({ category }: { category: string }) {
  const { data } = useEntityQuery<BlogPost>("blogPost", {
    filter: { 
      published: true,
      category: { eq: category }
    },
    sort: { field: "date", order: "desc" }
  })
  
  // Automatically re-subscribes when category changes
  // Automatically refetches when posts change
}
```

### Nested Queries (Future)
```typescript
function BlogPostWithComments({ postId }: { postId: string }) {
  const { data: post } = useEntityQuery<BlogPost>("blogPost", {
    filter: { id: { eq: postId } }
  })
  
  const { data: comments } = useEntityQuery<Comment>("blogPostComment", {
    filter: { 
      blog_post_id: { eq: postId },
      approved: true
    },
    sort: { field: "upvotes", order: "desc" },
    limit: 10
  })
  
  // System automatically detects relationship
  // Subscribes to both blogPost and blogPostComment changes
}
```

---

*Document Version: 1.0*  
*Last Updated: 2026-03-07*  
*Author: AI Assistant*
