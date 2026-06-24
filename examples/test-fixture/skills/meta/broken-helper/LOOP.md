## Entry Conditions

Story is ready

## Loop State Schema

| field | type |
|-------|------|

## Single Iteration Step

1. invoke the thing
2. perform some action
3. execute the task

## Proof of Progress

Tests pass

## Halt Conditions

halt unknown-reason after 100 iterations

## State Transition Rule

transition in-dev -> in-qa
  trigger done

transition broken-helper -> running-missing-skill
  trigger escalate

## Handoff Target

handoff broken-helper to unknown
