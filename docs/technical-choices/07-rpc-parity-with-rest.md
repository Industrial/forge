# Technical choices: Epic 7 — RPC parity with REST for entity operations

This document records technical decisions for the seventh deliverable (RPC parity with REST). It fixes how the same entity operations are exposed over a second transport and how auth and scope apply.

**Epic reference:** [07_rpc-parity-with-rest](../deliverables/07_rpc-parity-with-rest.md)

---

## 1. Same operations on both transports

- **Choice:** The same entity operations (list with query spec, get by id, create, update, delete) are available over **REST** and over a **second transport**. The second transport is **HTTP/2**. Request and response shapes are logically equivalent: same query spec, same payloads, same identifiers. No operation exists on one transport only (for entity CRUD).
- **Implication:** The generic handler logic (or a shared service layer) is transport-agnostic; REST and HTTP/2 RPC both invoke it. RPC is invoked via HTTP/2 (e.g. POST to an RPC endpoint) with a request body carrying the method, entity, params, and correlation id; the response body carries the result or error. The exact envelope (e.g. JSON object with `method`, `entity`, `params`, `id`) is implementation-defined but follows from HTTP/2 request/response semantics.

---

## 2. Authorization and scope identical

- **Choice:** Authorization and scope apply **identically** on both transports. The same entity permissions (Epic 2) and the same scope (from request context) are used to allow or deny each operation. There is no separate “RPC permission” or “RPC scope.”
- **Implication:** Identity (token) and scope (org id, role id) are supplied **per request** on the second transport (e.g. in each HTTP/2 request’s headers or in the RPC envelope). This allows scope switching without reconnecting.

---

## 3. Client choice of transport

- **Choice:** Clients may use either transport for any entity operation. Behavior and permissions are consistent; the client chooses based on latency, connection model, or other concerns.
- **Implication:** No operation is restricted to one transport. Documentation and client libraries should support both.

---

## 4. No embedded relations on RPC

- **Choice:** RPC responses follow the same contract as REST: no embedded relations, relations as IDs only, response columns per entity metadata (Epic 6). The same validation and error handling apply.
- **Implication:** List/get over RPC return the same shape as over REST. Expand/include are not supported and are rejected with an equivalent error (e.g. 400 or RPC error code).

---

## 5. Error handling parity

- **Choice:** Error conditions (404, 403, 400, 422, etc.) are represented on the second transport in an equivalent way (e.g. error code or structured error object) so that clients can handle them the same way as REST status codes.
- **Implication:** The RPC layer maps internal errors to a consistent error format; clients do not need transport-specific error handling for authz or validation.

---

## 6. Second transport: HTTP/2

- **Choice:** The second transport for entity RPC is **HTTP/2**. RPC requests are sent as HTTP/2 requests (e.g. POST with a body); responses are HTTP/2 responses with a body. Multiplexing and streams are available for concurrent RPCs and (in Epic 8) for subscription delivery.
- **Implication:** No WebSocket or other transport is required for Epic 7. Wire format is determined by HTTP/2: request/response with a body (e.g. JSON envelope containing method, entity, params, id). The same server can serve REST (e.g. HTTP/1.1 or HTTP/2) and the RPC endpoint over HTTP/2.

---

## 7. Identity and scope per request

- **Choice:** On the second transport (HTTP/2), **identity** (token) and **scope** (org id, role id) are sent **per request**—e.g. in each request’s headers (same as REST) or in the RPC envelope. They are not fixed at connection time.
- **Implication:** Clients can change scope between RPCs without reconnecting. Auth and scope are evaluated per RPC request; no connection-level session state for scope.

---

## 8. Subscriptions on the same transport

- **Choice:** **Subscribe** and **unsubscribe** (Epic 8) are RPC methods on the **same** transport (HTTP/2). One-shot entity RPCs and subscription lifecycle (subscribe, unsubscribe, and delivery of subscription updates) all use HTTP/2. How subscription updates are delivered (e.g. a long-lived stream, server push, or a dedicated stream endpoint) is defined in Epic 8.
- **Implication:** The wire format and connection model for Epic 7 should accommodate both request-response RPC and server-to-client push (or streamed responses) for subscriptions. No separate transport (e.g. WebSocket) is required for subscriptions.

---

## Summary table

| Topic | Choice |
|-------|--------|
| Operations | Same list/get/create/update/delete on REST and HTTP/2. |
| Second transport | HTTP/2; RPC via HTTP request/response with body. |
| Wire format | Determined by HTTP/2; envelope (method, entity, params, id) in body. |
| Auth and scope | Identical on both transports; per request on HTTP/2 (no connection-time scope). |
| Client choice | Either transport for any operation; consistent behavior. |
| Response shape | Same as REST; no embed, IDs only, column metadata (Epic 6). |
| Errors | Equivalent representation on second transport. |
| Subscriptions | Subscribe/unsubscribe and updates on same transport (HTTP/2); delivery mechanism in Epic 8. |
