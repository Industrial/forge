/**
 * Debug runner for Playwright tests
 * Uses Playwright's programmatic API to run tests and debug issues
 */
import { chromium } from '@playwright/test'
import { spawn } from 'node:child_process'
import { promisify } from 'node:util'

const _exec = promisify(spawn)

async function main() {
  console.log('🔍 Starting Playwright debug runner...')
  console.log('📁 Working directory:', process.cwd())

  try {
    // Try to import the config first
    console.log('\n1️⃣  Loading Playwright config...')
    const configModule = await import('./playwright.config.ts')
    const config = configModule.default
    console.log('✅ Config loaded:', {
      testDir: config.testDir,
      baseURL: config.use?.baseURL,
      projects: config.projects?.map((p) => p.name),
    })

    // Try to list tests using Playwright's CLI
    console.log('\n2️⃣  Listing tests using Playwright CLI...')
    const listProcess = spawn(
      'bunx',
      ['playwright', 'test', '--list', '--reporter=list'],
      {
        cwd: process.cwd(),
        stdio: 'pipe',
      },
    )

    let listOutput = ''
    let listError = ''

    listProcess.stdout?.on('data', (data) => {
      const text = data.toString()
      listOutput += text
      process.stdout.write(text)
    })

    listProcess.stderr?.on('data', (data) => {
      const text = data.toString()
      listError += text
      process.stderr.write(text)
    })

    // Wait for process with timeout
    const listPromise = new Promise<void>((resolve, reject) => {
      listProcess.on('close', (code) => {
        if (code === 0) {
          resolve()
        } else {
          reject(new Error(`Process exited with code ${code}`))
        }
      })
      listProcess.on('error', reject)
    })

    try {
      await Promise.race([
        listPromise,
        new Promise((_, reject) =>
          setTimeout(
            () => reject(new Error('TIMEOUT after 10 seconds')),
            10000,
          ),
        ),
      ])
      console.log('\n✅ Test listing completed')
      console.log('Output:', listOutput)
      if (listError) {
        console.log('Errors:', listError)
      }
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : String(error)
      console.log('\n❌ Test listing failed or timed out:', msg)
      console.log('Output so far:', listOutput)
      console.log('Errors so far:', listError)
      listProcess.kill('SIGTERM')
    }

    // Try to import a test file directly
    console.log('\n3️⃣  Attempting to import test file...')
    try {
      const testModule = await import('./tests/01-guest.spec.ts')
      console.log('✅ Test file imported successfully')
      console.log('Exports:', Object.keys(testModule))
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : String(error)
      const stack = error instanceof Error ? error.stack : undefined
      console.log('❌ Failed to import test file:', msg)
      console.log('Stack:', stack)
    }

    // Try to import helper modules
    console.log('\n4️⃣  Checking helper imports...')
    const helpers = [
      '../pages/LoginPage',
      '../fixtures/page-layers',
      '../helpers/expect',
      '../helpers/locator',
      '../fixtures/test-data',
      '../playwright.config',
    ]

    for (const helper of helpers) {
      try {
        const module = await import(helper)
        console.log(
          `✅ ${helper}: imported (${Object.keys(module).length} exports)`,
        )
      } catch (error: unknown) {
        const msg = error instanceof Error ? error.message : String(error)
        console.log(`❌ ${helper}: ${msg}`)
      }
    }

    // Try to create a browser instance
    console.log('\n5️⃣  Testing browser launch...')
    try {
      const browser = await chromium.launch({
        headless: true,
      })
      console.log('✅ Browser launched')

      const context = await browser.newContext()
      console.log('✅ Context created')

      const page = await context.newPage()
      console.log('✅ Page created')

      await page.goto('about:blank')
      console.log('✅ Navigation successful')

      await browser.close()
      console.log('✅ Browser closed')
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : String(error)
      const stack = error instanceof Error ? error.stack : undefined
      console.log('❌ Browser launch failed:', msg)
      console.log('Stack:', stack)
    }

    // Try to run a single test programmatically
    console.log('\n6️⃣  Attempting to run test programmatically...')
    try {
      await import('@playwright/test')
      console.log('✅ Playwright test API imported')

      // This won't work directly, but let's see what happens
      console.log('Note: Playwright tests need to be run via the CLI')
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : String(error)
      console.log('❌ Failed to import test API:', msg)
    }
  } catch (error: unknown) {
    const msg = error instanceof Error ? error.message : String(error)
    const stack = error instanceof Error ? error.stack : undefined
    console.error('\n💥 Fatal error:', msg)
    console.error('Stack:', stack)
    process.exit(1)
  }

  console.log('\n✅ Debug runner completed')
}

main().catch((error) => {
  console.error('💥 Unhandled error:', error)
  process.exit(1)
})
