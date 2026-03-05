# Technical choices: Epic 3 — Entity registry

This document records technical decisions for the third deliverable (entity registry). It fixes how the registry is defined, what it contains, and how it is used so that Epic 2 (permissions), Epic 4 (query spec), and Epic 5 (generic handler) have a single source of truth.

**Epic reference:** [03_entity-registry](../deliverables/03_entity-registry.md)

---

## 1. Single registry, single source of truth

- **Choice:** The system has exactly one registry of entities. That registry is the single source of truth for: (a) which permission keys exist (Epic 2: `<entity>.<action>` per registered entity), (b) which entities the API serves (Epic 5: list/get/create/update/delete), and (c) later, which entities can be subscribed to (Epic 8). No separate list or config for “permissions” vs “API entities”; both are derived from the registry.
- **Implication:** Any component that needs “the set of entities” or “allowed permission keys” or “allowed filter/sort fields” reads from this registry. Adding or removing an entity is a single change in one place; permission keys and API surface update accordingly without per-entity handler code or codegen.

---

## 2. Entity identifier convention

- **Choice:** The stable identifier for an entity is a **lowercase, snake_case** string (e.g. `user`, `org_role`, `organization`). It is used consistently in: permission keys (`<entity>.<action>`), API paths or request parameters (e.g. `/api/entities/user` or `entity=user`), and any client or config that refers to the entity. No mixed casing or alternate spellings in the same system.
- **Implication:** The registry keys (or primary id) use this convention. SeaORM or other internal model names may differ (e.g. `User` in code); the registry maps the public snake_case id to the internal model or table. This keeps URLs, permission keys, and docs consistent.

---

## 3. Metadata per entity

- **Choice:** Each entity in the registry has at least:
  - **id:** The stable identifier (snake_case, see §2).
  - **supported_actions:** Which of `create`, `read`, `update`, `delete` are supported for this entity. Default is all four unless specified otherwise (e.g. an audit log might be read-only).
- **Choice (optional metadata):** The registry may also hold, per entity:
  - **allowed_filter_fields:** Which fields may be used in the unified query spec for filtering (Epic 4). If absent, **derived from the entity’s model/schema** (e.g. all columns, or all non-sensitive columns).
  - **allowed_sort_fields:** Which fields may be used for sorting. If absent, **derived from the entity’s model/schema** in the same way.
  - **display_name** (or equivalent): For UI only; not used for auth or API contract.
- **Implication:** The generic handler (Epic 5) and query layer (Epic 4) read this metadata to validate requests and to build queries. Validation rules for create/update (e.g. required fields, types) are not required in the registry for Epic 3; they can be derived from the model/schema (Epic 5) or added later.

---

## 4. Registry population and mutability

- **Choice:** The registry is **populated at application startup** (or first use). It is **read-only at runtime**: no API or admin action can add or remove entities during a running session. Adding or removing an entity is a **deployment-time** change (e.g. update code, then deploy or restart). This matches “create entities, migrations, maybe seeds and you’re good to go” without requiring runtime admin UI for the registry.
- **Choice (storage):** The registry is **code-defined**: a list or map in code (e.g. one entry per entity) built at application startup. No config file or database table for the registry itself; adding an entity is done by adding a registration in code (alongside the persistence layer and migrations).
- **Implication:** The registry is an in-memory structure built from code at startup. No need for concurrent updates or versioning at runtime. Developers add entities by editing code (model + migration + one registry entry).

---

## 5. Relationship to persistence (SeaORM / tables)

- **Choice:** For this template, every registered entity is backed by a **single persistence abstraction** (today: a SeaORM entity / table). The registry associates each entity id with the information needed to perform CRUD (e.g. which SeaORM entity type or table name, and optionally which columns). “Virtual” or non-persisted entities are out of scope for Epic 3; they can be considered later if needed.
- **Implication:** The generic handler (Epic 5) will use the registry to look up, for a given entity id, how to execute list/get/create/update/delete. That implies a mapping from entity id to model/table; the exact mechanism (type map, enum, or table name + reflection) is an implementation detail.

---

## 6. Derivation of permission keys from the registry

- **Choice:** The set of **allowed permission keys** (for role assignment and validation) is derived strictly from the registry: for each entity, the four keys `<entity>.create`, `<entity>.read`, `<entity>.update`, `<entity>.delete` (or only those in that entity’s `supported_actions`), plus the wildcards `all.read` and `all.write`. No permission key exists that does not correspond to a registered entity (plus wildcards).
- **Implication:** Epic 2’s “source of the set of permission keys” is fully defined: it is the registry. APIs that list “available permissions” or validate `permission_key` on role-permission add/update must use this derived set.

---

## 7. Minimal onboarding (no codegen)

- **Choice:** Adding a new entity to the system requires: (1) defining the persistence (e.g. SeaORM model and migration), (2) **registering** the entity in the registry (one entry: id, supported_actions, and optional metadata). No generated per-entity handler code; no separate step to “add permissions” or “expose in API” beyond registration.
- **Implication:** The registry must be structured so that a single registration step (e.g. one line in a list, one config block, or one row in a table) is sufficient for the entity to appear in permission lists and in the generic API.

---

## 8. Error handling and unknown entities

- **Choice:** If a request (REST or later RPC) refers to an **unknown entity id** (not in the registry), the server responds with **404 Not Found** (or 400 Bad Request with a clear message that the entity is unknown). No silent fallback or default entity.
- **Choice:** If a component (e.g. role-permission assignment) receives an **invalid permission key** (not in the derived set from the registry), the behaviour is as in Epic 2 (e.g. 422 Unprocessable Entity). The registry is not queried at request time for “is this key valid”; the derived set is computed once (at startup or on registry load) and reused.

---

## 9. Filter/sort when not specified in registry

- **Choice:** When `allowed_filter_fields` or `allowed_sort_fields` are not specified for an entity in the registry, the system **derives** them from the entity’s model/schema (e.g. all columns, or all non-sensitive columns). No explicit allowlist is required for basic filter/sort support.
- **Implication:** Epic 4 and 5 can assume that every registered entity has a defined set of filterable and sortable fields (either from registry metadata or derived from the model). The derivation rule (e.g. which columns to exclude, if any) is an implementation detail.

---

## Summary table

| Topic | Choice |
|-------|--------|
| Registry count | Single registry; source of truth for permissions, API, and (later) subscriptions. |
| Entity id | Lowercase snake_case; used in permissions, API, and registry. |
| Metadata | At least id + supported_actions; optional filter/sort allowlists, display_name. |
| Registry storage | Code-defined; list or map in code, built at startup. |
| Mutability | Read-only at runtime; populated at startup; changes are deployment-time. |
| Persistence | Every entity backed by one persistence abstraction (e.g. SeaORM); no virtual entities in Epic 3. |
| Permission keys | Derived from registry (four actions per entity + all.read, all.write). |
| Filter/sort default | When not in registry, derive from entity’s model/schema. |
| Onboarding | Entity = persistence + one registry entry; no codegen, no per-entity handlers. |
| Unknown entity | 404 (or 400) when entity id is not in registry. |
