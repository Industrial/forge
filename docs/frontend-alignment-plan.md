# Frontend alignment plan

This document recommends how to align the frontend with the current backend (generic handler, model registry, RPC, subscriptions, change events) **without introducing new epic numbers**. Frontend work is treated as implementation of the same epics 1–10, with explicit frontend scope and acceptance criteria added where missing.

**Related:** [Migration path: legacy routes → generic entity handler](migration-legacy-routes-to-generic-entity-handler.md), deliverables 01–10, technical-choices 01–10.

---

## 1. Strategy

- **Reuse existing epics.** No Epic 11, 12, …; frontend tasks are part of Epics 1, 2, 5, 7, 8, and 10.
- **One source of truth for frontend decisions.** A single **Frontend implementation and choices** doc (or one per epic where needed) records how the client implements scope, permissions, generic API usage, RPC, subscribe/refetch, and E2E. This avoids drift and gives new contributors a clear contract.
- **Order work by dependency.** Scope (1) and permissions (2) underpin everything; generic entity UI (5) and RPC usage (7) can follow; subscribe + refetch (8) and E2E (10) come last so the full live path is validated.

---

## 2. Epics with frontend impact (and order)

| Order | Epic | Frontend scope | Depends on |
|-------|------|----------------|------------|
| 1 | **Epic 1** – Scoped session | Scope selector UX; persistence (e.g. localStorage); sending `X-Organization-Id`, `X-Role-Id` (and token) on every request; `needs_profile_select` → redirect to scope selection. | — |
| 2 | **Epic 2** – Entity-based permissions | Use resolved permissions to show/hide or disable UI (e.g. create button, delete action). Optional: permission-driven route guards. | 1 |
| 3 | **Epic 5** – Generic REST handler | Any UI that lists/gets/creates/updates/deletes via `GET/POST /api/entities/:entity_id` and `GET/PATCH/DELETE /api/entities/:entity_id/:id` with query spec (filter, sort, pagination). Migrate from legacy routes where desired. | 1, 2 |
| 4 | **Epic 7** – RPC parity | Use RPC for entity operations where beneficial (e.g. single connection, batching). Same auth/scope as REST. Optional if REST-only is enough. | 1, 2 |
| 5 | **Epic 8** – Subscriptions | Subscribe (e.g. `POST /api/rpc` with method `subscribe`, entity_id + params); open invalidation stream (e.g. SSE); on invalidation, refetch the affected query. | 1, 2, 4 or REST |
| 6 | **Epic 10** – End-to-end | E2E tests (e.g. Playwright): login → scope selection → list → subscribe → CUD → receive invalidation and refetch. | 1–5 |

Epics 3, 4, 6, 9 have no direct frontend deliverables (registry, query spec, no-embed contract, server-side matching).

---

## 3. Documents to produce

| Document | Purpose |
|----------|--------|
| **This plan** (`frontend-alignment-plan.md`) | Strategy, order, and open questions. |
| **Frontend implementation and choices** (`frontend-implementation-and-choices.md`) | Per-epic decisions and implementation notes (scope persistence, permission UI, generic API usage, RPC, subscribe/refetch, E2E). Filled in after answers to the questions below. |
| **Frontend task list (optional)** | Concrete backlog items derived from this plan; can live in this repo or in bd/issue tracker. |

No separate “new epics” doc; frontend is covered by extending the existing epic set with explicit frontend scope and the implementation doc.

---

## 4. Open questions per epic (for you to answer)

Before or while we write **Frontend implementation and choices**, the following need decisions. Answer per epic (or per question); I’ll incorporate your answers into the next document.

### Epic 1 – Scoped session

1. **Where is scope persisted today?** (e.g. localStorage, sessionStorage, React context only, URL?) Should we standardize on one approach?
2. **Is there already a dedicated scope/profile selection screen**, or should we add one and redirect multi-org users there after login when `needs_profile_select` is true?
3. **Where should the “current scope” be shown in the UI** (header, sidebar, profile menu), and should switching scope be one click or a modal/page?

### Epic 2 – Entity-based permissions

4. **How should the frontend obtain the list of effective permissions?** (e.g. from `GET /api/auth/me` with scope headers, or a dedicated permissions endpoint?) Is that already implemented?
5. **Preferred pattern for permission-driven UI:** hide disabled actions, show but disable, or show and let the API return 403 (and how to show errors)?

### Epic 5 – Generic REST handler

6. **Which screens/features should migrate to the generic entity API first?** (e.g. organization list/detail, then user, then role?) Or should we keep legacy routes as primary and use generic only for new features?
7. **Do you want a generic list/get UI component** (e.g. table with filter/sort/pagination driven by entity_id and query spec), or per-entity screens that happen to call the generic API?

### Epic 7 – RPC parity

8. **Is RPC (POST /api/rpc with entity.list, entity.get, …) required on the frontend**, or is REST-only acceptable for now? If RPC is required, do you already have (or want) a small RPC client helper?

### Epic 8 – Subscriptions

9. **How should the client open the invalidation stream?** (e.g. GET to an SSE endpoint, or another mechanism?) What is the exact URL/contract (e.g. `/api/subscriptions/stream`, query params, or RPC-based stream)?
10. **Refetch strategy:** on each invalidation event, refetch by subscription_id only, or by subscription_id + entity_id + query spec? Do you use a cache layer (e.g. React Query, SWR) that we should invalidate by key?
11. **Which views must be “live” first?** (e.g. organization list, dashboard widgets?) So we can prioritize subscribe/refetch for those.

### Epic 10 – End-to-end

12. **E2E stack:** Are Playwright and the existing test setup the right place for “login → scope → list → subscribe → CUD → invalidation/refetch” tests, or do you prefer a different tool/scope?
13. **Any specific flows** (e.g. “org admin creates org, second user in same org sees list update”) that must be covered by E2E first?

---

## 5. Next step

Once you’ve answered the questions above (per epic or in bulk), the next document to generate is **Frontend implementation and choices** (`docs/frontend-implementation-and-choices.md`), with one section per epic (1, 2, 5, 7, 8, 10) and your decisions recorded. Optionally we can then add a short **Frontend task list** (or bd issues) derived from this plan.

You can answer in any order (e.g. “Epic 1: 1. localStorage, 2. Yes we have a screen, 3. Header dropdown”); I’ll integrate answers and produce the implementation doc section by section if you prefer to do it epic by epic.
