# Analysis: Organization name change not syncing across browsers

## Summary

When admin@admin.com changed an organization name in one browser, the change did not sync to the other browser (multi@email.com, Default Editor scope) viewing the Organizations list. No requests were made in the multi@email.com window because **no invalidation was sent to that client**. Root cause: **scope-based matching** — the change event and the other client’s subscription use different `organization_id` values, so the subscription is not considered “affected” and never receives an invalidation.

There is **no server-side logging** when invalidations are published or delivered, so you would not see invalidation messages in server logs.

---

## Flow (current behavior)

1. **Subscribe (multi@email.com, Organizations page)**  
   - Frontend calls subscribe RPC with `entity_id: "organization"` and the request scope (e.g. Default Editor’s default org = Org B).  
   - Server creates a subscription with `SubscriptionMeta { entity_id: "organization", organization_id: scope.organization_id }` (Org B).  
   - That subscription is tied to multi@email.com’s SSE connection.

2. **Update (admin@admin.com)**  
   - Admin updates an organization (e.g. Org A) in the other browser.  
   - Request carries admin’s scope (e.g. Org A or another org).  
   - Generic handler calls `publish_change(ChangeEvent { model_id: "organization", resource_id, action: "update", organization_id: Some(scope.organization_id) })` — so the event’s `organization_id` is the **editor’s scope** (e.g. Org A).

3. **Matching**  
   - Change worker runs `notify_affected_by(event)`.  
   - `subscriptions_affected_by(event)` keeps only subscriptions where:
     - `entity_id == "organization"`, and  
     - `event.organization_id.is_none() || event.organization_id == Some(sub.meta.organization_id)`.  
   - Event has `organization_id: Some(Org A)`, subscription has `organization_id: Org B` → **no match** → no invalidation for multi@email.com’s subscription.

4. **Result**  
   - No `InvalidationEvent` is sent for multi@email.com’s subscription_id.  
   - Their SSE stream never gets that subscription_id, so the frontend never runs `trigger(subscription_id)` and never refetches.  
   - Hence “no requests were made” in the multi@email.com window.

So the other browser doesn’t sync because the server intentionally does not consider that subscription “affected” under the current scope rules.

---

## Target behavior (agreed)

- **Same query, different result sets:** Subscriptions are to a **query shape** (entity_id + params: pagination, sort, filter). The **result set** is determined at **read time** by the request’s scope. So “list organizations” is one logical query; admin gets all orgs, multi gets only orgs in their scope.
- **Read path:** The server must never return data the user cannot see. List (and get) for organizations must be scope-filtered: when the request has an org scope, return only that org (or only orgs the user is a member of).
- **Invalidation:** When any organization changes, invalidate **all** subscriptions watching the `"organization"` entity (platform-level event). Each client refetches; the server returns scope-filtered data. Multi may see no visible change (e.g. renamed org was not in their scope); that is correct.
- **Multi-tenancy:** An org-scoped user (e.g. multi@email.com) must not see other organizations in list or get.

---

## Implementation status and plan

### Done

| Item | Location | What was done |
|------|----------|----------------|
| **1. Platform-level invalidation for organization** | `generic_entity.rs` | For create/update/delete, when `entity_id == "organization"`, publish `ChangeEvent` with `organization_id: None` so all `"organization"` subscriptions are invalidated. Helper: `change_event_organization_id()`. |
| **2. Server-side tracing for invalidation** | `forge-live/src/subscription.rs` | `tracing::debug!` in `publish_change`, `notify_affected_by` (with `affected_count`), and `send_invalidation`. Use `RUST_LOG=debug` or `forge_live=debug` to see flow. |

### Remaining: scope-filtered read path for organization

Today the **list** (and **get**) path for the `"organization"` entity does **not** apply scope. Both REST and RPC return all organizations to any user who has `organization.read`, regardless of request scope. So multi@email.com can currently see all orgs; they should only see the org(s) in their scope.

**Responsibility:** The read path (REST and RPC) must filter so that when the request has an org scope (`scope.organization_id == Some(org_id)`), list returns only that org and get returns 404 for any other org id.

---

## Concrete plan: scope-filtered organization list and get

### 1. Scope-filter list result for organization (app layer)

- **Where:** Template app: `handlers/generic_entity.rs` and, for RPC, the call site of `list_entity_with_spec` in `handlers/rpc.rs`.
- **Idea:** After obtaining the list payload from `registry::list_models`, if `entity_id == "organization"` and `scope.organization_id.is_some()`, filter the `"data"` array to only include the object whose `"id"` equals `scope.organization_id`. No changes to `RestModel` or the db crate.
- **Steps:**
  1. Add a helper, e.g. `apply_organization_list_scope(value: &mut serde_json::Value, scope: &RequestScope)`: if `scope.organization_id` is `Some(org_id)`, get `data` array from `value`, retain only items with `id == org_id.to_string()`, write back (and optionally adjust any `total`/`next_cursor` if present to match).
  2. In **REST** `list_entities`: after the `registry::list_models` success branch, call `apply_organization_list_scope(&mut v, &scope)` when `entity_id == "organization"`.
  3. In **RPC** `entity.list`: ensure the list path has access to scope (it already has `scope` in the handler). Change `list_entity_with_spec` to accept an optional `scope: Option<&RequestScope>`, and after `registry::list_models` in that function (or in a wrapper used by both REST and RPC), call `apply_organization_list_scope` when `entity_id == "organization"` and `scope.is_some()`. Wire the RPC handler to pass `Some(&scope)` into `list_entity_with_spec`.

### 2. Scope-filter get-by-id for organization (app layer)

- **Where:** Template app: `handlers/generic_entity.rs` (`get_entity_by_id`).
- **Idea:** After a successful `registry::get_model` for `entity_id == "organization"`, if `scope.organization_id.is_some()` and the returned resource’s `id` is not equal to `scope.organization_id`, return 404 (so org-scoped users cannot fetch other orgs by id).
- **Steps:**
  1. In `get_entity_by_id`, after getting `Some(value)` from `get_model` for `entity_id`, if `entity_id == "organization"` and `scope.organization_id == Some(org_id)` and `value.get("id").and_then(|v| v.as_str()) != Some(org_id.to_string().as_str())`, return 404 response instead of the value.

### 3. Tests

- **List:** Integration or unit: with an org-scoped request (e.g. `x-organization-id: <org_a>`), list `"organization"` returns only the row with `id == org_a`; with no org scope (or platform scope), list returns all organizations (existing behavior).
- **Get:** With org-scoped request, get organization `<org_b>` when `org_b != scope.organization_id` returns 404; get for `scope.organization_id` returns 200 and the org.

### 4. Optional: document “platform vs org scope” for organization

- In `docs/technical-choices/09-subscription-matching-and-delivery.md` or a small “organization entity” note: organization list is one logical query; invalidation is platform-level (all subscribers notified); data returned is scope-dependent (platform = all orgs, org scope = that org only).

---

## Beads epic and tasks (bd swarm)

Epic **forge-itcz** with small, parallelizable tasks. Run `bd ready` for unblocked work; `bd swarm` to consume in parallel.

| ID | Task | Deps | Unit/Integration |
|----|------|------|------------------|
| **forge-itcz.1** | Add `apply_organization_list_scope` helper + unit test | — | Unit (helper) |
| **forge-itcz.2** | Wire helper in REST `list_entities` | .1 | — |
| **forge-itcz.3** | Add scope param to `list_entity_with_spec`, apply filter | .1 | — |
| **forge-itcz.4** | RPC `entity.list` pass scope into `list_entity_with_spec` | .3 | — |
| **forge-itcz.5** | Scope-filter `get_entity_by_id` for organization | — | — |
| **forge-itcz.6** | Integration test: list orgs scoped vs platform | .2, .4 | Integration |
| **forge-itcz.7** | Integration test: get org 404/200 when scoped | .5 | Integration |
| **forge-itcz.8** | Docs: organization platform vs org scope | — | — |

**First wave (parallel):** forge-itcz.1, forge-itcz.5, forge-itcz.8.  
**Second wave:** forge-itcz.2, forge-itcz.3 (after .1).  
**Third wave:** forge-itcz.4 (after .3).  
**Fourth wave:** forge-itcz.6, forge-itcz.7 (after list and get impl).

---

## References

- `SubscriptionStore::subscriptions_affected_by` — scope rule: `event.organization_id.is_none() || event.organization_id == Some(record.meta.organization_id)` (`crates/forge-live/src/subscription.rs`).
- `generic_entity.rs` — `change_event_organization_id()`, `publish_change` with `organization_id: None` for organization; list/get currently do not filter by scope.
- `list_entity_with_spec` — currently takes `(db, entity_id, spec)`; no scope; used by RPC `entity.list`.
- `rpc.rs` — `rpc_subscribe` sets `SubscriptionMeta { organization_id: scope.organization_id }`; RPC list has `scope` but does not pass it to list.
- `db/src/registry.rs` — `list_models(model_id, db, spec)`; no scope parameter; `Organization::list(db, spec)` returns all rows.
- `docs/technical-choices/09-subscription-matching-and-delivery.md` — scope-aware matching; platform-level = `organization_id.is_none()` matches all subs for that model.
