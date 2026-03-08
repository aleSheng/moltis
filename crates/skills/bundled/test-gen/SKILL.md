---
name: test-gen
description: "Generate unit tests for existing code with high coverage"
license: Apache-2.0
metadata:
  author: moltis
  version: "1.0"
allowed-tools: Bash Read Write Edit Grep Glob
---

# Test Generator

Generate comprehensive unit tests for existing functions, methods, or modules.

## When to use

Use this skill when the user asks to generate tests, add test coverage, or write unit tests for specific code.

## Steps

1. Read the target file(s) to understand the code under test.
2. Identify the testing framework already in use:
   - Rust: built-in `#[cfg(test)]` module, look for existing test patterns
   - JavaScript/TypeScript: look for jest, vitest, mocha, or playwright configs
   - Python: look for pytest, unittest patterns
3. Analyze each public function/method for:
   - Happy path scenarios
   - Edge cases (empty input, boundary values, max/min)
   - Error cases (invalid input, missing data, network failures)
   - Null/None/undefined handling
4. Generate tests following the project's existing test conventions:
   - Match naming style (`test_*`, `it("should ...")`, etc.)
   - Use the same assertion library
   - Follow AAA pattern: Arrange, Act, Assert
5. Place tests in the correct location:
   - Rust: `#[cfg(test)] mod tests` in the same file, or `tests/` directory
   - JS/TS: co-located `*.test.ts` or `__tests__/` directory
6. Run the tests to verify they pass.

## Rules

- Test behavior, not implementation details
- Each test should test one thing and have a clear name
- Do not mock what you do not own unless necessary
- Prefer real values over random/generated test data
- Include both positive and negative test cases
