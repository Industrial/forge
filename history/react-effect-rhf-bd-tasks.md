# React + Effect-TS + RHF: Beads Task List

Plan: move default frontend template to React + Effect-TS + RHF using **plain Effect.ts** (Effect.runPromise at boundary, no effect-atom).

## Dependency graph

```
                    forge-n1p (Add npm deps)
                           │
         ┌─────────────────┼─────────────────┬─────────────────┐
         ▼                 ▼                 ▼                 ▼
    forge-92p          forge-wg6          forge-y19         forge-eha
 (Schema fragments)  (effectSchemaResolver) (FormField)   (runPromise pattern)
         │                 │                 │                 │
         └────────┬────────┴────────┬────────┘                 │
                  ▼                 │                          ▼
             forge-6gr              │                    forge-8e5
    (Migrate UsersPage forms)       │            (Users list as Effect)
                  │                 │                          │
                  ├────────────────┼──────────────────────────┤
                  ▼                 ▼                          ▼
             forge-jjq          forge-f9h                  (forge-f9h)
    (Remaining forms)    (Replace fetch with runPromise)
                  │
                  ▼
             forge-h4y
    (Document patterns)
```

## Tasks (in execution order)

| Order | ID       | Title                                                                 | Deps (blocked by) |
|-------|----------|-----------------------------------------------------------------------|-------------------|
| 1     | **forge-n1p**  | Add npm deps: effect, @effect/schema, react-hook-form                 | — |
| 2a    | **forge-92p**  | Create shared Effect Schema fragments (email, password, org/role)      | forge-n1p |
| 2b    | **forge-y19**  | Create/refactor FormField primitives for RHF                          | forge-n1p |
| 2c    | **forge-eha**  | Use Effect.runPromise for data fetching and side effects in components | forge-n1p |
| 3     | **forge-wg6**  | Implement effectSchemaResolver(schema) for RHF                         | forge-n1p, forge-92p |
| 4     | **forge-6gr**  | Migrate UsersPage add/edit forms to RHF + Effect Schema                | forge-92p, forge-wg6, forge-y19 |
| 5     | **forge-8e5**  | Create users list fetch as Effect and call via runPromise              | forge-eha |
| 6     | **forge-f9h**  | Replace UsersPage fetch with Effect.runPromise in useEffect            | forge-8e5, forge-6gr |
| 7     | **forge-jjq**  | Migrate remaining forms to RHF + Effect Schema                       | forge-6gr |
| 8     | **forge-h4y**  | Document form and Effect patterns (README or docs)                      | forge-jjq |

## Ready to start

- **forge-n1p** is the only task with no dependencies. Run:
  ```bash
  devenv shell -- bd update forge-n1p --status in_progress
  ```

## Commands

- List open: `devenv shell -- bd list --status=open`
- Ready (unblocked): `devenv shell -- bd ready`
- Show one: `devenv shell -- bd show forge-n1p`
- Claim: `devenv shell -- bd update <id> --status in_progress`
- Close: `devenv shell -- bd close <id> --reason "Done"`
