# Feature Template

Use this template for feature specifications.

This file is for implementation-ready decisions, not discussion history.

## Writing Rules

- Use explicit language: `must`, `must not`, `only`, `exactly`, `at most`, `at least`.
- Do not write narrative recap or negotiation history.
- If something is not decided, put it under `Open Questions`.
- If something is intentionally deferred, put it under `Out of Scope`.
- If the future architecture matters, define the boundary now, but specify only the behavior that will actually be implemented.
- Delete sections that do not apply. Do not leave placeholder ambiguity in the final feature file.

## Status

- `draft` | `implementation-ready`

If any item remains in `Open Questions`, the status must stay `draft`.

## Dependencies

List prerequisite planning documents or implementation steps that must be completed first.

Use one of:

- `None`
- `<dependency 1>`
- `<dependency 2>`

Dependencies describe execution order. A document may still be `implementation-ready` as a specification while remaining blocked on unfinished dependencies.

## Goal

State the outcome in one short paragraph.

Then list the concrete user-visible or system-visible results:

- `<result 1>`
- `<result 2>`

## In Scope

- `<implemented behavior 1>`
- `<implemented behavior 2>`

## Out of Scope

- `<explicitly not implemented 1>`
- `<explicitly not implemented 2>`

## Core Concepts

Define the domain entities and roles this feature introduces or depends on.

### `<concept 1>`

`<exact definition>`

### `<concept 2>`

`<exact definition>`

## State Model

List the important states and transitions if the feature has lifecycle behavior.

- `<state or invariant 1>`
- `<transition rule 1>`

## Event Model

List the source-of-truth events, if the feature is event-driven.

It must record:

- `<event 1>`
- `<event 2>`

## Naming and Identity Rules

Only include this section if names, IDs, uniqueness, or matching rules matter.

- `<identity rule 1>`
- `<identity rule 2>`

## Behavior Rules

This is the main policy section. Split by actor or concern.

### `<actor or concern 1>`

- `<rule 1>`
- `<rule 2>`

### `<actor or concern 2>`

- `<rule 1>`
- `<rule 2>`

## Concurrency and Runtime Rules

Only include this section if execution model matters.

- `<single-writer / queue / blocking / async rule>`
- `<ordering / parallelism / backpressure rule>`

## Provider or Integration Boundary

Only include this section if the feature depends on pluggable local/remote providers or external systems.

- `<boundary rule 1>`
- `<boundary rule 2>`

## Protocol and Data Shapes

Only include this section if the feature defines concrete HTTP, signaling, event-log, or artifact payloads.

- `<request or event shape 1>`
- `<response or artifact shape 1>`

## Delivery to Clients

Specify exactly what is sent to clients and over which channel.

- `<data or media item 1>`
- `<transport or channel rule 1>`

## Persistence and Source of Truth

Specify what is stored, where the truth lives, and what may be derived.

- `<source of truth rule 1>`
- `<persistence rule 1>`

## Output Artifacts

Only include this section if the feature generates durable outputs such as minutes, reports, files, or records.

### `<artifact name>`

- `<required field 1>`
- `<required field 2>`

## Acceptance Criteria

The feature is complete only when all of the following are true:

- `<verifiable criterion 1>`
- `<verifiable criterion 2>`
- `<verifiable criterion 3>`

## Open Questions

Leave this section empty in implementation-ready specs.

- `<unresolved decision 1>`
- `<unresolved decision 2>`
