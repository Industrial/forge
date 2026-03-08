#!/usr/bin/env bash
set -euo pipefail

# Script to process e2e test fix tasks one by one
# Usage: bin/process-e2e-tasks.sh [max-tasks]

MAX_TASKS="${1:-10}"
PROCESSED=0

echo "Processing up to $MAX_TASKS e2e test fix tasks..."

while [ $PROCESSED -lt $MAX_TASKS ]; do
    # Get first ready task
    TASK_JSON=$(cd /data/Code/rust/forge && devenv shell -- bd ready --json 2>&1 | tail -100 | python3 -c "
import sys, json
try:
    tasks = json.load(sys.stdin)
    if tasks and len(tasks) > 0:
        task = tasks[0]
        print(json.dumps(task))
    else:
        sys.exit(1)
except:
    sys.exit(1)
" 2>/dev/null || echo "")

    if [ -z "$TASK_JSON" ]; then
        echo "No more ready tasks!"
        break
    fi

    TASK_ID=$(echo "$TASK_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['id'])" 2>/dev/null)
    TASK_TITLE=$(echo "$TASK_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['title'])" 2>/dev/null)
    TEST_SPEC=$(echo "$TASK_JSON" | python3 -c "
import sys, json, re
desc = json.load(sys.stdin)['description']
match = re.search(r\"bin/test-e2e-single '([^']+)'\", desc)
if match:
    print(match.group(1))
" 2>/dev/null || echo "")

    if [ -z "$TEST_SPEC" ]; then
        echo "Could not extract test spec from task $TASK_ID, skipping..."
        # Mark as done anyway to avoid infinite loop
        cd /data/Code/rust/forge && devenv shell -- bd close "$TASK_ID" --reason "Could not extract test spec" --json >/dev/null 2>&1 || true
        PROCESSED=$((PROCESSED + 1))
        continue
    fi

    echo ""
    echo "=========================================="
    echo "[$((PROCESSED + 1))/$MAX_TASKS] Processing: $TASK_TITLE"
    echo "Task ID: $TASK_ID"
    echo "Test: $TEST_SPEC"
    echo "=========================================="

    # Claim task
    cd /data/Code/rust/forge && devenv shell -- bd update "$TASK_ID" --status in_progress --claim --json >/dev/null 2>&1 || true

    # Run test to see failure
    echo "Running test to see current failure..."
    cd /data/Code/rust/forge/crates/forge-cli/templates/default
    TEST_OUTPUT=$(timeout 120 bin/test-e2e-single "$TEST_SPEC" 2>&1 || true)

    # Check if test passed
    if echo "$TEST_OUTPUT" | grep -q "passed\|PASS"; then
        echo "✅ Test passed! Closing task..."
        cd /data/Code/rust/forge && devenv shell -- bd close "$TASK_ID" --reason "Test now passes" --json >/dev/null 2>&1 || true
        PROCESSED=$((PROCESSED + 1))
        continue
    fi

    # Extract error from test output
    ERROR_MSG=$(echo "$TEST_OUTPUT" | grep -A 10 "Error\|FAIL\|not found\|toBeVisible" | head -20 || echo "Unknown error")
    echo "Test failed. Error summary:"
    echo "$ERROR_MSG" | head -10

    echo ""
    echo "⚠️  Manual intervention needed for this test."
    echo "   Please fix the application code and re-run:"
    echo "   bin/test-e2e-single '$TEST_SPEC'"
    echo ""
    echo "   Then close the task with:"
    echo "   devenv shell -- bd close $TASK_ID --reason 'Fixed' --json"
    echo ""

    PROCESSED=$((PROCESSED + 1))

    # Ask if we should continue (for manual mode)
    if [ "${AUTO_CONTINUE:-}" != "1" ]; then
        read -p "Press Enter to continue to next task, or Ctrl+C to stop..."
    fi
done

echo ""
echo "Processed $PROCESSED tasks."
