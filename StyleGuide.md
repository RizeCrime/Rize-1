# Style Guide

All of the below generally applies to all branches, but frequent exceptions are made for refactor and/or dev branches.

## General Principles

- **Clarity is Key:** Write code that is easy to understand and maintain. If it works and is readable, it's generally acceptable.
- **Follow-ability:** The code itself should also be able to serve as a Learning Resource. Code that is easy to follow and understand for a beginner may be preferable to 'Idiomatic' code. (Leave a `### Dev Metadata` comment with your reasoning above.)
- **Use `cargo fmt`:** Ensure all code is formatted using `cargo fmt` before committing.

## Me-Specific Guideline Additions

- **Minimize Comments:** Strive to write self-documenting code by using clear variable and function names. Avoid comments that merely restate the code.
- **Internal Function Metadata:** For information about a function that is primarily relevant for internal understanding (e.g., implementation details, rationale), use the last section of the comment, prefixed with `### Dev Metadata`.
- **Avoid Deep Nesting:** Reduce code complexity by avoiding excessive nesting and indentation. Prefer using guard clauses to handle edge cases early and flatten the code structure.
