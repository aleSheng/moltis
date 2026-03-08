---
name: refactor
description: "Refactor code to improve readability and maintainability without changing behavior"
license: Apache-2.0
metadata:
  author: moltis
  version: "1.0"
allowed-tools: Bash Read Write Edit Grep Glob
---

# Code Refactor

Improve code structure, readability, and maintainability while preserving exact behavior.

## When to use

Use this skill when the user asks to refactor, clean up, simplify, or restructure code.

## Steps

1. Read the target code and understand its current behavior completely.
2. Run existing tests to establish a baseline: `cargo test` / `npm test` / `pytest`.
3. Identify refactoring opportunities:
   - Extract repeated code into shared functions
   - Simplify complex conditionals with guard clauses
   - Replace manual loops with iterators/combinators where clearer
   - Reduce function length (aim for single responsibility)
   - Improve naming to better express intent
   - Remove dead code
4. Apply changes incrementally, one refactoring at a time.
5. After each change, run tests to verify behavior is preserved.
6. Summarize what was changed and why.

## Rules

- Never change observable behavior
- If no tests exist, write them first before refactoring
- Prefer small, reviewable changes over large rewrites
- Do not add features or fix bugs during refactoring
- Keep the diff minimal and focused
