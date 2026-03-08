---
name: explain
description: "Explain code step-by-step in a beginner-friendly way"
license: Apache-2.0
metadata:
  author: moltis
  version: "1.0"
allowed-tools: Read Grep Glob
---

# Code Explainer

Explain code clearly, step by step, suitable for developers at any level.

## When to use

Use this skill when the user asks to explain code, understand a function, or asks "what does this do?".

## Steps

1. Read the target code completely, including imports and type definitions it depends on.
2. Provide a **one-sentence summary** of what the code does at a high level.
3. Walk through the code section by section:
   - Explain each logical block (not every line)
   - Describe **what** it does and **why** it does it that way
   - Highlight non-obvious patterns, idioms, or tricks
   - Explain any domain-specific terminology
4. If the code uses external libraries or APIs, briefly explain what those do.
5. End with:
   - **Key takeaways**: The most important things to understand
   - **Potential gotchas**: Subtle behavior or edge cases to be aware of

## Rules

- Adjust depth to the user's apparent experience level
- Use analogies for complex concepts when helpful
- Do not just restate the code in English; explain the reasoning
- If the code has bugs or issues, mention them but keep focus on explanation
- Keep explanations structured with clear headings
