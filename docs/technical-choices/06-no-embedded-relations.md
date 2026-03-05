# Technical choices: Epic 6 — No embedded relations in API responses

This document records technical decisions for the sixth deliverable (no embedded relations). It fixes how list/get responses are shaped, how relations are represented, and how response columns are defined.

**Epic reference:** [06_no-embedded-relations](../deliverables/06_no-embedded-relations.md)

---

## 1. No embed or expand

- **Choice:** List and get responses return **only** the requested entity’s data (one “table”/resource). The API does **not** support embedding or expanding related entities. There is no `expand`, `include`, or equivalent parameter.
- **Implication:** The handler never joins or loads related entities for inclusion in the response. Clients that need related data issue separate list/get requests (e.g. filtered by the foreign key).

---

## 2. Reject expand and include with 400

- **Choice:** If a client sends `expand`, `include`, or any equivalent parameter (in query string or body), the server responds with **400 Bad Request** and a clear message that expand/include are not supported. The server does not ignore the parameter; it rejects the request.
- **Implication:** Clients must not send expand/include. Document this in the API contract so that clients do not rely on ignored parameters.

---

## 3. Relations as identifiers only

- **Choice:** Relationships are represented in the response **only** by their **identifiers** (e.g. foreign key columns such as `org_id`, `user_id`). The response does **not** include hypermedia links (e.g. `user_url`) or any other relation metadata beyond the id. IDs only.
- **Implication:** Response shape is a flat object (or array of flat objects for list) with the entity’s own columns; relation columns are just id values. Clients use those ids to issue follow-up requests if needed.

---

## 4. Response column metadata (same file as entity)

- **Choice:** Which columns are returned in list and get responses is defined by **metadata** for that entity. This metadata is defined **in the same file as the entity** (i.e. alongside the entity’s registry entry or model definition). For example: an allow-list of column names to return, or an exclude-list (e.g. never return `password_hash`), or a default “all columns except excluded.” The exact shape (allow vs exclude, or both) is implementation-defined; the rule is that the definition lives with the entity.
- **Implication:** The generic handler (or serialization layer) consults this metadata when building list/get responses. Columns not in the allow-list (or in the exclude-list) are omitted. This allows per-entity control over sensitive or internal columns without embedding relations. The entity registry (Epic 3) or the code that defines the entity holds this metadata.

---

## Summary table

| Topic | Choice |
|-------|--------|
| Embed/expand | Not supported; no nested related entities in responses. |
| expand / include param | Rejected with 400 Bad Request. |
| Relations in response | Identifiers only; no relation links or URLs. |
| Response columns | Metadata in same file as entity; defines which columns are returned (allow-list, exclude-list, or equivalent). |
