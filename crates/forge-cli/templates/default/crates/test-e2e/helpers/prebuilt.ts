import fs from 'node:fs'
import path from 'node:path'
import { PREBUILT_ROOT } from '../playwright.config.js'

export function getPrebuiltRoot(): string {
  return PREBUILT_ROOT
}

export function prebuiltExists(): boolean {
  return fs.existsSync(PREBUILT_ROOT)
}

export function assertProjectLayout(): void {
  const root = PREBUILT_ROOT
  const required = [
    'Cargo.toml',
    'crates/app/src/main.rs',
    'crates/db/src/lib.rs',
    'config/app.toml',
    'config/db.toml',
    '.gitignore',
  ]
  for (const p of required) {
    const full = path.join(root, p)
    if (!fs.existsSync(full)) {
      throw new Error(`prebuilt project missing: ${p} (run bin/test-e2e first)`)
    }
  }
  const mainRs = fs.readFileSync(
    path.join(root, 'crates/app/src/main.rs'),
    'utf-8',
  )
  const libRsPath = path.join(root, 'crates/app/src/lib.rs')
  const hasAppNewInMain = mainRs.includes('App::new()')
  const hasAppNewInLib =
    fs.existsSync(libRsPath) &&
    fs.readFileSync(libRsPath, 'utf-8').includes('App::new()')
  if (!hasAppNewInMain && !hasAppNewInLib) {
    throw new Error('main.rs or crates/app/src/lib.rs should use App::new()')
  }
  if (
    !mainRs.includes('.serve()') &&
    !mainRs.includes('into_router_before_state')
  ) {
    throw new Error('main.rs should call .serve() or into_router_before_state')
  }
}

export function assertAuthLayout(): void {
  const root = PREBUILT_ROOT
  const authRs = path.join(root, 'crates/db/src/auth.rs')
  const orgRs = path.join(root, 'crates/db/src/models/organization.rs')
  const membershipRs = path.join(root, 'crates/db/src/models/membership.rs')
  const userRs = path.join(root, 'crates/db/src/models/user.rs')
  const handlersAuth = path.join(root, 'crates/app/src/handlers/auth.rs')
  const mainRsPath = path.join(root, 'crates/app/src/main.rs')

  if (!fs.existsSync(authRs)) {
    throw new Error('crates/db/src/auth.rs missing')
  }
  if (!fs.existsSync(orgRs)) {
    throw new Error('organization.rs missing')
  }
  if (!fs.existsSync(membershipRs)) {
    throw new Error('membership.rs missing')
  }

  const userModel = fs.readFileSync(userRs, 'utf-8')
  if (
    !userModel.includes('current_org_id') ||
    !userModel.includes('current_role')
  ) {
    throw new Error('user model should have current_org_id and current_role')
  }
  if (!userModel.includes('impl AuthzContext')) {
    throw new Error('user model should impl AuthzContext')
  }

  const authHandlers = fs.readFileSync(handlersAuth, 'utf-8')
  const hasAdmin =
    authHandlers.includes('admin') &&
    (authHandlers.includes('is_admin') ||
      authHandlers.includes('record_authz_denied'))
  if (!hasAdmin) {
    throw new Error(
      'auth handlers should gate admin on is_admin or record_authz_denied',
    )
  }

  const mainRs = fs.readFileSync(mainRsPath, 'utf-8')
  const libRsPath = path.join(root, 'crates/app/src/lib.rs')
  const authSource = fs.existsSync(libRsPath)
    ? fs.readFileSync(libRsPath, 'utf-8')
    : mainRs
  if (
    !authSource.includes('post_route') ||
    !authSource.includes('/api/auth/admin')
  ) {
    throw new Error(
      'app (main.rs or lib.rs) should have post_route and /api/auth/admin',
    )
  }
  if (mainRs.includes('forge::prelude')) {
    throw new Error('main.rs should not use forge::prelude')
  }
}

export function readFile(p: string): string {
  return fs.readFileSync(path.join(PREBUILT_ROOT, p), 'utf-8')
}

export function pathExists(p: string): boolean {
  return fs.existsSync(path.join(PREBUILT_ROOT, p))
}
