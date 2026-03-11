# typescript-setup

Shared TypeScript base config for the monorepo. All TypeScript projects extend this so options stay consistent. No project references; type-check runs per project with `tsgo --noEmit -p <path>` (no emit).

**Usage:** In any `tsconfig.json`:

- From repo root: `"extends": "./packages/typescript-setup/tsconfig.base.json"`
- From `packages/<name>`: `"extends": "../typescript-setup/tsconfig.base.json"`
- From template frontend: `"extends": "../../../../../packages/typescript-setup/tsconfig.base.json"`
- From template test-e2e: `"extends": "../../../../../../packages/typescript-setup/tsconfig.base.json"`

Then add project-specific options (e.g. `include`, `exclude`, `paths`, `types`).
