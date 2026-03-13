# Refactor 1

## Status

- `draft`

## Dependencies

- `None`

## Goal

Rename the current `room` model to `meeting` across the app and backend without changing runtime behavior.

After this refactor, the system must still do what it does today:

- list joinable meetings
- join a meeting
- send audio to the backend
- transcribe finalized speech on the backend
- log transcripts on the backend

This file is a refactor specification, not a feature description.

## In Scope

- public API terminology
- backend domain terminology
- backend route naming
- WebSocket signaling field naming
- mobile app UI terminology
- code identifiers where practical

## Out of Scope

- agent participants
- moderator behavior
- owner behavior
- agenda support
- minutes generation
- changes to meeting behavior
- changes to media behavior
- changes to transcription behavior

## Core Concepts

### Meeting

`Meeting` replaces the current concept currently named `Room`.

This is a model rename only. It must not add new behavior.

### Meeting Participant

A meeting participant is the same runtime concept currently represented by a joined peer connection in the room.

This refactor must not change participant behavior.

## Naming and Identity Rules

- `room` must be renamed to `meeting` in public API and feature-facing terminology.
- `room_id` must be renamed to `meeting_id` in public API and feature-facing terminology.
- Existing internal implementation details may be migrated incrementally, but the end state of this refactor must not expose `room` terminology in the public surface.

## Behavior Rules

### No Functional Change

- The refactor must not change who can join.
- The refactor must not change media flow.
- The refactor must not change transcription flow.
- The refactor must not change what is logged on the backend except for renamed terminology if applicable.

### Join Flow

- A client that can currently join a room must be able to join the renamed meeting after the refactor.
- The join flow must continue to establish signaling and media exactly as before.

## Delivery to Clients

- The HTTP route currently exposed as `/rooms` must become `/meetings`.
- Any payload field currently named `room_id` in public API or signaling must become `meeting_id`.
- Any user-visible app text that currently refers to rooms must refer to meetings.

## Persistence and Source of Truth

- This refactor must not introduce new persistence.
- Existing in-memory state remains the source of truth.

## Acceptance Criteria

The refactor is complete only when all of the following are true:

- the app can list meetings instead of rooms
- the app can join a meeting instead of a room
- the app can still send audio after joining
- the backend still accepts the joined session and signaling flow
- backend transcription still works
- backend transcript logging still works
- public routes, payload fields, and user-visible strings use `meeting` terminology instead of `room`
- no new product behavior has been introduced

## Open Questions

- Should this refactor rename internal Rust type names and module names fully in the same step, or is it sufficient to rename only the public surface first?
