---
name: debug
description: "Systematically debug errors using root-cause analysis"
license: Apache-2.0
metadata:
  author: moltis
  version: "1.0"
allowed-tools: Bash Read Grep Glob
---

# Systematic Debugger

Diagnose and fix bugs through structured root-cause analysis.

## When to use

Use this skill when the user reports a bug, error, test failure, or unexpected behavior.

## Steps

1. **Reproduce**: Understand and reproduce the issue.
   - Get the exact error message, stack trace, or unexpected output
   - Identify the minimal reproduction steps
   - Run the failing command/test to confirm the issue

2. **Locate**: Find where the error originates.
   - Search for the error message in the codebase
   - Trace the call stack from the error site upward
   - Identify the exact line where behavior diverges from expectation

3. **Understand**: Analyze why the code fails.
   - Read the surrounding code and understand the intended logic
   - Check recent changes with `git log -p --follow <file>` if relevant
   - Identify the root cause (not just the symptom)

4. **Fix**: Apply the minimal correct fix.
   - Change only what is necessary to fix the root cause
   - Consider edge cases the fix might affect
   - Do not refactor unrelated code

5. **Verify**: Confirm the fix works.
   - Re-run the failing test/command
   - Run the full test suite to check for regressions
   - Explain the root cause and fix to the user

## Rules

- Always reproduce before fixing
- Fix the root cause, not the symptom
- One bug, one fix (do not bundle unrelated changes)
- If the fix is uncertain, explain the hypothesis before applying it
