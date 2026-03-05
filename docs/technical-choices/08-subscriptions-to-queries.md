# Technical choices: Epic 8 — Subscriptions to queries

This document records technical decisions for the eighth deliverable (subscriptions to queries). It fixes how clients subscribe, how updates are delivered over HTTP/2, and what is sent in an update.

**Epic reference:** [08_subscriptions-to-queries](../deliverables/08_subscriptions-to-queries.md)

**Implementation:** Subscribe RPC parses and validates params (filter, sort, order, offset, limit) as **ListQuerySpec** per model using `effective_filter_fields` and `effective_sort_fields`; invalid params → 400. The structured spec is stored in `SubscriptionMeta.params` (JSON-serialized ListQuerySpec) for use by Epic 9 matching.

---

## 1. Subscribe to a query (same spec as list)

- **Choice:** A client subscribes by sending a **query** in the same form as the unified query specification (Epic 4): entity plus filter, sort, and pagination. The subscribe request is an RPC on the same transport (HTTP/2); the payload includes the query. Authorization and scope apply: the client must have `<entity>.read` in the current scope to subscribe. Invalid query or missing permission → 400 or 403.
- **Implication:** Subscribe uses the same validation as list (allowed filter/sort fields, pagination bounds). Scope is taken from the request context (per Epic 7, per request); the subscription is bound to that scope.

---

## 2. Delivery mechanism: long-lived stream over HTTP/2

- **Choice:** Subscription updates are delivered over a **long-lived stream** on the same transport (HTTP/2). The client opens a stream (e.g. via a dedicated endpoint or RPC that establishes a stream); the server sends events on that stream when the result set of a subscribed query may have changed. No separate transport (e.g. WebSocket) is used.
- **Implication:** The server holds the stream (or stream id) per client and pushes events as they occur. Connection/stream lifecycle (reconnect, backpressure) is part of the implementation. Epic 9 will define how change events are matched to subscriptions and written to the stream.

### 2.1 HTTP/2 transport; SSE as stream format

- **Choice:** We use **HTTP/2** for subscription delivery. The stream is a long-lived HTTP response on the same transport as RPC. When the client uses HTTP/2, the stream is an HTTP/2 response stream. We do **not** use WebSocket or a second protocol for subscriptions.
- **Wire format:** The implementation uses **Server-Sent Events (SSE)**, `Content-Type: text/event-stream`, as the format for events on that HTTP/2 stream. SSE is a standard, widely supported way to deliver a unidirectional stream over HTTP/2; it reuses the same connection and does not require a separate transport. Clients should connect over HTTP/2 when available so the subscription stream and RPC share one connection.

---

## 3. Update payload: invalidation hint only (simplicity and safety)

- **Choice:** When a change affects a subscription’s result set, the server sends an **invalidation hint** only: e.g. “subscription X may have changed, refetch.” The client is responsible for refetching the query (e.g. via list RPC or REST) to get the new data. No full snapshot and no delta in the push for this epic; invalidation only. Delta or snapshot can be added later.
- **Implication:** Simple and safe: the server does not need to compute deltas or re-run the query to build a snapshot; it only signals “this subscription was affected.” The client refetches and replaces its local view. Reduces server complexity and avoids subtle bugs from incremental patches.

---

## 4. Subscription identity: server-assigned id

- **Choice:** The **subscription id** is **server-assigned**. The server returns a unique subscription id in the response to subscribe. The client uses that id to **unsubscribe** and receives it with each invalidation so it can associate the hint with the right subscription. The client does not assign ids.
- **Implication:** Subscribe response includes at least the subscription id. Unsubscribe takes that id. Invalidation messages include the subscription id so the client can route the hint to the correct view or refetch the correct query.

---

## 5. One update per subscription

- **Choice:** When a single data change affects multiple subscriptions, the matching/delivery layer (Epic 9) sends **one update per affected subscription**. Each message is subscription-scoped (includes the subscription id and is self-contained). No batching of multiple subscription ids into one message for this epic unless we later optimize; one invalidation per subscription keeps the contract simple.
- **Implication:** A change that affects N subscriptions results in N invalidation messages (one per subscriber per subscription). Epic 9 implements matching and then iterates over affected subscriptions to deliver.

---

## 6. Staleness and page boundaries: invalidation only

- **Choice:** For this epic we do **not** compute or send deltas (e.g. “row X updated”, “row Y left the page”). Any change that might affect a subscription’s result set results in a single **invalidation hint** for that subscription. The client refetches the full query result (or the current page). Delta-based updates can be considered in a later epic.
- **Implication:** No server-side logic for “row moved in/out of page” or minimal patch; only “this subscription was affected, refetch.”

---

## 7. No limit on concurrent subscriptions per client

- **Choice:** There is **no** enforced limit on the number of concurrent subscriptions per client (or per connection) for this epic. If resource or abuse concerns arise later, a limit and appropriate response (e.g. 429) can be added.
- **Implication:** The server accepts subscribe requests as long as the client is authenticated and authorized; subscription storage and matching scale with total subscriptions (Epic 9).

---

## Summary table

| Topic | Choice |
|-------|--------|
| Subscribe | Same query spec as list; RPC on HTTP/2; auth and scope per request. |
| Delivery | Long-lived stream over HTTP/2; SSE (`text/event-stream`) as wire format; server pushes events. |
| Update payload | Invalidation hint only; client refetches. No delta/snapshot in push for this epic. |
| Subscription id | Server-assigned; returned on subscribe; used for unsubscribe and in invalidation. |
| One change, many subscriptions | One invalidation message per affected subscription. |
| Staleness / page | Invalidation only; no delta; client refetches. |
| Multiple subscriptions per client | No limit. |
