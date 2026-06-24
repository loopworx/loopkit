## Entry Conditions

Story is in-dev with clear ACs and development environment ready

## Loop State Schema

| field | type | description |
|-------|------|-------------|
| story | str  | Story identifier |
| ac_index | int | Current acceptance criteria index |
| test_status | bool | Whether current test passes |

## Single Iteration Step

1. trigger Pick next AC from the story
2. Write a failing test for the AC
3. Run the test and confirm it fails
4. Implement the minimum code to make the test pass
5. Run the test and confirm it passes
6. Refactor the implementation while keeping tests green
7. Commit the changes

## Proof of Progress

`cargo test` output shows green for all existing tests and the new test.

## State Transition Rule

transition in-dev → in-deskcheck
  trigger all ACs implemented and green
  handoff running-desk-checks to qa-agent

transition in-dev → halted-stall
  halt stall after 10 iterations

transition in-deskcheck → in-qa
  trigger all ACs approved by QA
  handoff running-regression-suite to qa-agent

transition in-deskcheck → in-dev
  trigger QA found bugs in AC
  handoff running-tdd-loops to developer

transition in-qa → in-acceptance
  trigger all regression tests pass
  handoff approving-stories to po-agent

transition in-qa → in-dev
  trigger QA found bugs
  handoff running-tdd-loops to developer

transition in-acceptance → in-dev
  trigger PO/UX found issues
  handoff running-tdd-loops to developer

transition in-acceptance → done
  trigger both PO and UX approved
  handoff done to all-agents

## Halt Conditions

halt stall after 10 iterations

## Handoff Target

handoff running-desk-checks to qa-agent
