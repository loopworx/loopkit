---
name: running-tdd-loops
description: Drives TDD cycles by writing failing tests first then implementing until green
---
# running-tdd-loops

## Description

This skill drives test-driven development cycles. It writes a failing test first, watches it fail, then implements the minimum code to make it pass, and finally refactors.

## Rules

1. Always write the test first
2. Never write implementation before seeing a failing test
3. Refactor only when all tests are green
4. Commit after each red-green-refactor cycle

## State Model

The development flow moves through these states:

- `in-dev` — developer is actively working on code
- `in-deskcheck` — QA reviews the acceptance criteria
- `in-qa` — full quality assurance check
- `in-acceptance` — PO/UX reviews the completed work
- `done` — story is complete

## Entry Conditions

- Story must have clear acceptance criteria
- Development environment must be configured
- All dependencies must be available

## Halt Conditions

- halt stall after 10 iterations
- halt human-gate when external approval is needed

For the full state machine contract, see [LOOP.md](LOOP.md).
