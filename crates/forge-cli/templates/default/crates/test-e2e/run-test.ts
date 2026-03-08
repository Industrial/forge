/**
 * Programmatic test runner using Playwright's CLI with detailed debugging
 * Spawns Playwright process and captures all output to debug hangs
 */
import { spawn } from 'node:child_process'
import { createInterface } from 'node:readline'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

async function runPlaywrightTest(testFile: string) {
  console.log('🔍 Starting Playwright test runner...')
  console.log('📁 Working directory:', process.cwd())
  console.log('📄 Test file:', testFile)
  console.log('⏰ Starting at:', new Date().toISOString())

  const testPath = path.isAbsolute(testFile)
    ? testFile
    : path.join(__dirname, 'tests', testFile)

  console.log('📂 Full path:', testPath)

  // Check if file exists
  try {
    const fs = await import('node:fs/promises')
    const stats = await fs.stat(testPath)
    console.log('✅ Test file exists:', stats.size, 'bytes')
  } catch (error: any) {
    console.error('❌ Test file not found:', error.message)
    process.exit(1)
  }

  // Spawn Playwright process with detailed output
  console.log('\n🚀 Spawning Playwright process...')
  const proc = spawn(
    'bunx',
    [
      'playwright',
      'test',
      testPath,
      '--reporter=list',
      '--workers=1',
      '--timeout=30000',
    ],
    {
      cwd: __dirname,
      stdio: ['pipe', 'pipe', 'pipe'],
      env: {
        ...process.env,
        DEBUG: 'pw:api,pw:browser,pw:protocol',
        NODE_OPTIONS: '--trace-warnings',
      },
    },
  )

  let stdout = ''
  let stderr = ''
  let hasOutput = false

  // Capture stdout
  proc.stdout?.on('data', (data) => {
    const text = data.toString()
    stdout += text
    hasOutput = true
    process.stdout.write(`[STDOUT] ${text}`)
  })

  // Capture stderr
  proc.stderr?.on('data', (data) => {
    const text = data.toString()
    stderr += text
    hasOutput = true
    process.stderr.write(`[STDERR] ${text}`)
  })

  // Log process events
  proc.on('spawn', () => {
    console.log('✅ Process spawned (PID:', proc.pid, ')')
  })

  proc.on('error', (error) => {
    console.error('\n❌ Process error:', error.message)
    console.error('Stack:', error.stack)
  })

  // Set up timeout
  const timeout = setTimeout(() => {
    console.error(
      '\n⏱️  TIMEOUT: Process has been running for 30 seconds with no output',
    )
    console.error('This suggests Playwright is hanging during test discovery')
    console.error('\n📊 Process info:')
    console.error('   PID:', proc.pid)
    console.error('   Exit code:', proc.exitCode ?? 'still running')
    console.error('   Signal:', proc.signalCode ?? 'none')
    console.error('\n📝 Output so far:')
    console.error('STDOUT:', stdout || '(empty)')
    console.error('STDERR:', stderr || '(empty)')
    console.error('\n💡 Possible causes:')
    console.error('   1. Circular import in test files')
    console.error('   2. Top-level code blocking execution')
    console.error('   3. Missing dependency causing silent failure')
    console.error('   4. Playwright waiting for server that never responds')

    proc.kill('SIGTERM')
    setTimeout(() => {
      if (!proc.killed) {
        console.error('\n⚠️  Process did not terminate, force killing...')
        proc.kill('SIGKILL')
      }
    }, 2000)
  }, 30000)

  // Wait for process to complete
  proc.on('close', (code, signal) => {
    clearTimeout(timeout)
    console.log('\n✅ Process completed')
    console.log('   Exit code:', code)
    console.log('   Signal:', signal)
    console.log('   Had output:', hasOutput)

    if (!hasOutput && code === null) {
      console.error('\n⚠️  Process exited without output - likely hung')
    }

    if (code !== 0) {
      console.error('\n❌ Process exited with error')
      console.error('STDOUT:', stdout || '(empty)')
      console.error('STDERR:', stderr || '(empty)')
      process.exit(code ?? 1)
    } else {
      console.log('\n✅ Tests completed successfully')
      console.log('STDOUT:', stdout)
      if (stderr) {
        console.log('STDERR:', stderr)
      }
    }
  })

  // Periodic heartbeat
  let heartbeatCount = 0
  const heartbeat = setInterval(() => {
    heartbeatCount++
    if (heartbeatCount % 5 === 0) {
      console.log(
        `💓 Heartbeat: ${heartbeatCount * 2}s elapsed, process still running (PID: ${proc.pid})`,
      )
      if (!hasOutput && heartbeatCount > 5) {
        console.log('⚠️  No output received yet - process may be hanging')
      }
    }
  }, 2000)

  proc.on('close', () => {
    clearInterval(heartbeat)
  })
}

// Main
const testFile = process.argv[2] || '01-guest.spec.ts'
runPlaywrightTest(testFile).catch((error) => {
  console.error('💥 Fatal error:', error)
  process.exit(1)
})
