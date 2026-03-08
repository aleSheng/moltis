---
name: commit
description: "Analyze staged changes and generate a conventional commit message"
license: Apache-2.0
metadata:
  author: moltis
  version: "1.0"
allowed-tools: Bash Read Grep Glob
---

# Smart Commit

Generate a well-structured conventional commit message by analyzing staged changes.

## When to use

Use this skill when the user asks to commit changes, create a commit, or asks for a commit message.

## Steps

1. Run `git diff --cached --stat` to see which files are staged. If nothing is staged, check `git status` and suggest what to stage.
2. Run `git diff --cached` to read the full diff of staged changes.
3. Run `git log --oneline -10` to understand the repository's commit message style and conventions.
4. Analyze the changes:
   - Determine the type: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`
   - Identify the scope from the changed files/modules
   - Summarize the **why** not the **what**
5. Generate a commit message following conventional commits format:
   ```
   type(scope): concise description

   Optional body explaining motivation and context.
   ```
6. Present the message to the user for approval before committing.
7. If approved, run `git commit -m "<message>"`.

## Rules

- Keep the subject line under 72 characters
- Use imperative mood ("add" not "added")
- Do not include file lists in the commit message
- If changes span multiple unrelated concerns, suggest splitting into multiple commits
- Never use `--no-verify` or skip hooks unless explicitly asked
