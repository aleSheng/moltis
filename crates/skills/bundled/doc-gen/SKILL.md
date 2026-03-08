---
name: doc-gen
description: "Generate documentation for functions, modules, and APIs"
license: Apache-2.0
metadata:
  author: moltis
  version: "1.0"
allowed-tools: Read Write Edit Grep Glob
---

# Documentation Generator

Generate clear, accurate documentation for code.

## When to use

Use this skill when the user asks to document code, add docstrings, generate API docs, or explain a module.

## Steps

1. Read the target code thoroughly to understand its purpose, inputs, outputs, and side effects.
2. Determine the documentation format for the language:
   - Rust: `///` doc comments with `# Examples` sections
   - Python: Google-style or NumPy-style docstrings (match existing convention)
   - JavaScript/TypeScript: JSDoc `/** */` comments
3. For each public item, document:
   - **Purpose**: What it does in one sentence
   - **Parameters**: Name, type, and meaning of each parameter
   - **Returns**: What is returned and when
   - **Errors**: When and why it can fail
   - **Examples**: A minimal usage example (when helpful)
   - **Panics/Safety** (Rust): Document panic conditions and unsafe invariants
4. Write documentation that adds value beyond what the type signature already tells you.
5. Do not document private implementation details unless they are complex.

## Rules

- Match the project's existing documentation style
- Be concise: one sentence is better than a paragraph when it suffices
- Document the **why** and **when**, not just the **what**
- Include examples only when the usage is non-obvious
- Do not add `@param` tags that just repeat the parameter name
