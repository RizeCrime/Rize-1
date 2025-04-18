# Style Guide

## General Principles

- **Clarity is Key:** Write code that is easy to understand and maintain. If it works and is readable, it's generally acceptable.
- **Use `cargo fmt`:** Ensure all code is formatted using `cargo fmt` before committing.

## Me-Specific Guideline Additions

- **Minimize Comments:** Strive to write self-documenting code by using clear variable and function names. Avoid comments that merely restate the code.
- **Internal Function Metadata:** For information about a function that is primarily relevant for internal understanding (e.g., implementation details, rationale), use the last section of the comment, prefixed with `### Dev Metadata`.
- **Avoid Deep Nesting:** Reduce code complexity by avoiding excessive nesting and indentation. Prefer using guard clauses to handle edge cases early and flatten the code structure.
