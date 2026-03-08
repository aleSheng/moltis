---
name: code-review
description: "Review code changes for bugs, security issues, performance, and readability"
license: Apache-2.0
metadata:
  author: moltis
  version: "1.0"
allowed-tools: Bash Read Grep Glob
---

# Code Review

Perform a thorough code review on changed files or a pull request diff.

## When to use

Use this skill when the user asks for a code review, PR review, or wants feedback on their changes.

## Steps

1. Identify the changes to review:
   - If a PR number or branch is mentioned, run `git diff main...HEAD` (or the appropriate base branch)
   - Otherwise, run `git diff` for unstaged or `git diff --cached` for staged changes
2. Read the full diff and understand the context by reading surrounding code in modified files.
3. Evaluate each change across these dimensions:

### Correctness
- Logic errors, off-by-one bugs, null/undefined handling
- Edge cases not covered
- Race conditions in concurrent code

### Security
- Input validation and sanitization
- SQL injection, XSS, command injection risks
- Secrets or credentials in code
- SSRF or path traversal vulnerabilities

### Performance
- Unnecessary allocations or copies
- N+1 query patterns
- Missing indexes or inefficient data structures
- Blocking calls in async context

### Readability
- Clear naming and intent
- Appropriate abstractions (not over- or under-engineered)
- Missing or misleading comments

4. Present findings organized by severity: **Critical** > **Warning** > **Suggestion**
5. For each finding, include the file, line, and a concrete fix suggestion.

## Rules

- Be specific and actionable, not vague
- Praise good patterns alongside pointing out issues
- Do not suggest style changes unless they affect readability significantly
