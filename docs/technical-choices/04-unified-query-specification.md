# Technical choices: Epic 4 — Unified query specification (filter, sort, pagination)

This document records technical decisions for the fourth deliverable (unified query specification). It fixes the shape of the query spec so that list operations (REST and later WebSocket subscribe) use the same contract.

**Epic reference:** [04_unified-query-specification](../deliverables/04_unified-query-specification.md)

---

## 1. Single contract for REST and WebSocket

- **Choice:** The same query specification (filter, sort, pagination) is used for: (a) REST list operations (one-shot), and (b) WebSocket subscribe (Epic 8). The structure is identical so that a client can express “list users with filter X, sort Y, page Z” in the same way over REST and in a subscription.
- **Implication:** One canonical representation of the query spec (e.g. a JSON-friendly structure). REST may expose it via query params and/or body; WebSocket subscribe carries it in the message payload. Parsing and validation logic are shared.

---

## 2. Filter representation and operators

- **Choice:** Filter is expressed as a structured set of conditions. Each condition is: **field** + **operator** + **value** (or no value for `is_null`). The exact wire format (query string vs JSON body) is implementation-defined; the logical structure is:
  - **Operators:** `eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `in`, `contains`, `starts_with`, `ends_with`, `is_null`.
  - **Field:** Must be one of the entity’s allowed filter fields (from registry or derived from model).
  - **Value:** Type-appropriate; for `in`, a list of values; for `is_null`, no value or a boolean.
- **Implication:** The handler (and later subscription matcher) validates that the field is in the entity’s allowed filter set and that the operator is supported for that field type. Unknown or disallowed field → 400 or 422 with a clear error.

---

## 3. Sort: single key for now

- **Choice:** Sort is specified by **one** sort key (field + direction). Direction is `asc` or `desc`. Multi-sort may be added later; for this epic, one key only.
- **Implication:** The sort field must be one of the entity’s allowed sort fields (from registry or derived). Invalid or disallowed field → 400 or 422. When sort is omitted, the server may use a default (e.g. primary key or `created_at`); the default is implementation-defined but must be documented or stable.

---

## 4. Pagination: offset+limit and cursor

- **Choice:** Pagination supports both:
  - **Offset + limit:** `offset` (non-negative integer) and `limit` (positive integer). Default **limit 20**, maximum **limit 100**. When omitted, default values apply.
  - **Cursor-based:** A cursor value (opaque to the client) and `limit`. Same default and max limit. Cursor is returned in the response so the client can request the next page.
- **Implication:** The API accepts either offset+limit or cursor+limit (not both in the same request). Invalid values (e.g. negative offset, limit &gt; 100) → 400 or 422 with a clear error. Response includes enough information to continue pagination (e.g. next cursor or total count, if applicable).

---

## 5. Scope separate from filter; scope overrides filter

- **Choice:** **Scope** (org/role from request context, e.g. headers) is **not** part of the query specification. Scope is applied by the handler from the request context. The **filter** in the query spec refers only to the entity’s own columns (allowed filter fields). When applying the query, **scope is applied first** (e.g. restrict to rows belonging to the request’s org); **filter is applied after** and may not contradict or override scope. If the client could somehow pass scope-like criteria in the filter, the server **overrides** them with the authoritative scope from context.
- **Implication:** List and subscribe never trust “scope” from the query body or params; scope comes only from auth/headers. Filter is validated against allowed fields and cannot be used to escape scope.

---

## 6. Error handling and defaults

- **Choice:**  
  - **Unknown or disallowed filter/sort field:** 400 Bad Request or 422 Unprocessable Entity with a clear message (e.g. “invalid filter field” or “field not sortable”).  
  - **Invalid pagination:** Negative offset, limit &lt; 1, or limit &gt; max (100) → 400 or 422 with a clear message.  
  - **Invalid operator or value type:** 400 or 422.  
  - **Omitted filter/sort/pagination:** Defaults apply: no filter (all rows within scope), default sort (implementation-defined), default limit 20 and offset 0 (or first page for cursor). Defaults are documented and stable.
- **Implication:** The spec defines defaults and validation rules so that clients and the subscription matcher behave predictably.

---

## Summary table

| Topic | Choice |
|-------|--------|
| Contract | Same query spec for REST list and WebSocket subscribe. |
| Filter | Field + operator + value; operators: eq, ne, gt, gte, lt, lte, in, contains, starts_with, ends_with, is_null. |
| Sort | Single sort key (field + asc/desc); default when omitted. |
| Pagination | Offset+limit and cursor+limit; default limit 20, max 100. |
| Scope | Separate from filter; from request context only; scope overrides any scope-like filter. |
| Errors | Invalid field → 400/422; invalid pagination → 400/422; documented defaults when omitted. |
