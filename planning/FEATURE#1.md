# Feature 1

## Status

- `implementation-ready`

## Dependencies

- [`REFAC#1.md`](./REFAC#1.md) must be completed before this feature is implemented.

## Goal

Implement meetings with human and agent participants where:

- humans speak in the meeting
- the backend transcribes finalized human utterances
- agents observe the meeting utterance stream and may speak according to explicit floor rules
- agent speech is generated as text, converted to audio, and sent back to clients
- when the meeting ends, the moderator generates written minutes from the agenda and meeting log

This file is a specification, not a discussion recap.

## In Scope

- meetings with humans and agents
- one owner per meeting
- one implicitly created moderator agent per meeting
- initial invited participants at meeting creation
- finalized human utterance transcription
- agent turn-taking rules
- local or remote model providers behind the same backend abstraction
- text-to-speech for agent replies
- written minutes generation at meeting end

## Out of Scope

- dynamic participant invitation or join during the meeting
- SFU injection of synthesized agent audio
- partial-transcript-triggered agent behavior
- interruption management for humans
- streaming token-by-token or chunk-by-chunk agent playback
- persistent storage beyond in-memory meeting state

## Core Concepts

### Meeting

A meeting has:

- exactly one owner
- exactly one moderator
- zero or more human participants
- zero or more non-moderator agent participants
- an ordered agenda
- a meeting event log

### Owner

The owner is the human or agent that creates the meeting.

The owner:

- provides the meeting agenda
- invites the initial participants
- is the only participant allowed to end the meeting

### Moderator

The moderator is an agent role that is implicitly created when the meeting is created.

The moderator:

- is a first-class agent participant
- has the fixed name `Moderator`
- uses system-defined instructions and a system-defined `voice_id`
- observes every finalized utterance
- controls agent floor allocation
- may speak for moderation purposes
- generates the written minutes after the meeting ends

### Participants

Participants are humans or agents.

All participants observe the same meeting utterance stream.

Non-moderator agents are floor-controlled. Humans are not.

### Agenda

The agenda is an ordered list of short phrases provided by the owner at meeting creation.

Example:

- `review launch timeline`
- `decide hiring priorities`
- `assign follow-up tasks`

### Meeting Event Log

The meeting event log is the source of truth for the meeting.

## State Model

- Meetings have exactly two states: `active` and `ended`.
- Newly created meetings start in state `active`.
- Meetings may transition only from `active` to `ended`.
- Once a meeting is `ended`, no new utterances, floor requests, or speaking turns may be accepted.

## Event Model

It must record:

- meeting creation
- participant list at creation
- finalized human utterances
- agent utterances
- agent floor requests
- moderator floor grant/deny decisions
- meeting end
- generated minutes

Minutes must be generated from the agenda and meeting event log, not from ad hoc model memory.

- `sequence_number` in the meeting event log must be strictly increasing within a meeting.

## Naming and Identity Rules

- Every agent participant has exactly one name.
- Agent names must be unique within a meeting.
- Agent names must be single-word non-empty trimmed strings.
- The moderator also has a name.

## Behavior Rules

### Human Speech

- Humans may speak freely.
- Humans do not need moderator permission to speak.
- Human overlap and interruption are tolerated.
- The backend acts on finalized utterance events, not on an idealized real-time turn model.

### Human Utterance Processing

- Only finalized human utterances may trigger agent behavior.
- Partial transcripts must not trigger agent behavior.
- Every finalized human utterance must be appended to the meeting utterance stream.

### Agent Speech Eligibility

A non-moderator agent may speak only if at least one of these is true:

1. it was directly addressed by name in a finalized utterance
2. it requested the floor and the moderator granted it

The moderator may speak when needed for moderation duties.

### Direct Address

- A direct address occurs when a finalized utterance contains an agent name as a case-insensitive whole-token match after stripping leading and trailing ASCII punctuation from transcript tokens.
- Direct-address matching considers only agent names.
- If multiple agent names appear in one utterance, the moderator chooses at most one of them to speak next.
- If no agent name appears, no non-moderator agent is directly addressed.

### Floor Requests

- After every finalized utterance, every non-moderator agent privately evaluates whether it wants the floor.
- A floor request is an internal meeting event, not spoken output.
- The moderator evaluates the current floor requests and grants the floor to at most one agent.
- If the moderator grants no request, no non-moderator agent speaks.
- If an agent is directly addressed, it is eligible to speak without first requesting the floor.

## Delivery to Clients

In the initial implementation:

- agent text is sent to clients over the signaling channel
- agent audio is sent to clients as a generated audio clip for local playback
- synthesized agent audio is not injected into the SFU

## Concurrency and Runtime Rules

- Only one agent response may be active at a time.
- Agents do not interrupt an active agent response.
- If the owner ends the meeting, all pending floor requests are discarded.
- local in-process inference and local TTS synthesis must not run directly on Tokio executor threads
- blocking local inference work must run behind blocking worker boundaries
- remote API model calls may be awaited directly through Tokio

## Provider or Integration Boundary

- The backend orchestration must be independent of the underlying model provider.
- It must support local in-process model and TTS execution for early development.
- It must support remote API-backed model and TTS execution later.
- The meeting orchestration layer must not depend on whether providers are local or remote.

## Protocol and Data Shapes

### Conventions

- All identifiers are UUID strings.
- All timestamps are RFC3339 UTC strings.
- Signaling messages continue to use the existing JSON envelope with optional `request_id`.
- Client signaling requests that expect direct responses must include `request_id`.
- Server responses to client signaling requests must echo the same `request_id`.
- Unsolicited server signaling events must omit `request_id`.

### HTTP `POST /meetings`

Request:

```json
{
  "title": "Quarterly Product Review",
  "owner": {
    "kind": "human",
    "name": "Frederic"
  },
  "invited_participants": [
    {
      "kind": "human",
      "name": "Alice"
    },
    {
      "kind": "agent",
      "name": "CTO",
      "instructions": "You are the CTO. Focus on technical risk, sequencing, and staffing.",
      "voice_id": "alloy"
    }
  ],
  "agenda": [
    "review launch timeline",
    "decide hiring priorities",
    "assign follow-up tasks"
  ]
}
```

Owner and invited participants use these tagged shapes:

```json
{
  "kind": "human",
  "name": "Alice"
}
```

```json
{
  "kind": "agent",
  "name": "CTO",
  "instructions": "You are the CTO. Focus on technical risk, sequencing, and staffing.",
  "voice_id": "alloy"
}
```

Validation rules:

- `title` must be a non-empty trimmed string.
- `agenda` must contain at least one item.
- Every agenda item must be a non-empty trimmed string.
- An entry in `invited_participants` must not have the same `kind` and `name` as `owner`.
- For agent participants, `instructions` must be a non-empty trimmed string.
- For agent participants, `voice_id` must be a non-empty trimmed string.
- Every agent name must be unique within the meeting, including the implicit moderator.
- The implicit moderator is system-created and must not be included in the request.

Response:

```json
{
  "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
  "title": "Quarterly Product Review",
  "state": "active",
  "owner_participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
  "moderator_participant_id": "fa0d8ef6-c7f9-4d2c-a935-f4cc1e1cc5b0",
  "participants": [
    {
      "participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
      "kind": "human",
      "meeting_role": "owner",
      "name": "Frederic"
    },
    {
      "participant_id": "fa0d8ef6-c7f9-4d2c-a935-f4cc1e1cc5b0",
      "kind": "agent",
      "meeting_role": "moderator",
      "name": "Moderator"
    },
    {
      "participant_id": "5f3afccf-6dd6-486e-9a24-12e11384165f",
      "kind": "human",
      "meeting_role": "participant",
      "name": "Alice"
    },
    {
      "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
      "kind": "agent",
      "meeting_role": "participant",
      "name": "CTO"
    }
  ],
  "agenda": [
    {
      "agenda_item_id": "92b0bc3b-7d14-4c46-96df-c01f35d9202a",
      "phrase": "review launch timeline"
    },
    {
      "agenda_item_id": "04515abc-1f84-4f53-af4d-5ef95a857b58",
      "phrase": "decide hiring priorities"
    },
    {
      "agenda_item_id": "81025425-2035-4058-8a65-dc2e0dc2e5bf",
      "phrase": "assign follow-up tasks"
    }
  ],
  "started_at": "2026-03-13T10:15:30Z",
  "ended_at": null
}
```

The response shape above is `MeetingSnapshot`.

### HTTP `GET /meetings`

Response:

```json
[
  {
    "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
    "title": "Quarterly Product Review",
    "state": "active",
    "owner_participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
    "moderator_participant_id": "fa0d8ef6-c7f9-4d2c-a935-f4cc1e1cc5b0",
    "participants": [
      {
        "participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
        "kind": "human",
        "meeting_role": "owner",
        "name": "Frederic"
      },
      {
        "participant_id": "fa0d8ef6-c7f9-4d2c-a935-f4cc1e1cc5b0",
        "kind": "agent",
        "meeting_role": "moderator",
        "name": "Moderator"
      },
      {
        "participant_id": "5f3afccf-6dd6-486e-9a24-12e11384165f",
        "kind": "human",
        "meeting_role": "participant",
        "name": "Alice"
      },
      {
        "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
        "kind": "agent",
        "meeting_role": "participant",
        "name": "CTO"
      }
    ],
    "agenda": [
      {
        "agenda_item_id": "92b0bc3b-7d14-4c46-96df-c01f35d9202a",
        "phrase": "review launch timeline"
      },
      {
        "agenda_item_id": "04515abc-1f84-4f53-af4d-5ef95a857b58",
        "phrase": "decide hiring priorities"
      },
      {
        "agenda_item_id": "81025425-2035-4058-8a65-dc2e0dc2e5bf",
        "phrase": "assign follow-up tasks"
      }
    ],
    "started_at": "2026-03-13T10:15:30Z",
    "ended_at": null
  }
]
```

`GET /meetings` returns an array of `MeetingSnapshot`.

### Signaling Client Messages

Join request:

```json
{
  "request_id": 1,
  "type": "join_meeting",
  "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
  "participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be"
}
```

End meeting request:

```json
{
  "request_id": 2,
  "type": "end_meeting",
  "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87"
}
```

Validation rules:

- `join_meeting` must fail if `meeting_id` does not exist.
- `join_meeting` must fail if `participant_id` does not belong to `meeting_id`.
- `end_meeting` must fail unless the requesting participant is the owner of the meeting.

### Signaling Server Messages

Join response:

```json
{
  "request_id": 1,
  "type": "meeting_joined",
  "peer_id": "0a92e284-f534-4308-ae28-35ff65230f5d",
  "participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
  "meeting": {
    "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
    "title": "Quarterly Product Review",
    "state": "active",
    "owner_participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
    "moderator_participant_id": "fa0d8ef6-c7f9-4d2c-a935-f4cc1e1cc5b0",
    "participants": [
      {
        "participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
        "kind": "human",
        "meeting_role": "owner",
        "name": "Frederic"
      },
      {
        "participant_id": "fa0d8ef6-c7f9-4d2c-a935-f4cc1e1cc5b0",
        "kind": "agent",
        "meeting_role": "moderator",
        "name": "Moderator"
      },
      {
        "participant_id": "5f3afccf-6dd6-486e-9a24-12e11384165f",
        "kind": "human",
        "meeting_role": "participant",
        "name": "Alice"
      },
      {
        "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
        "kind": "agent",
        "meeting_role": "participant",
        "name": "CTO"
      }
    ],
    "agenda": [
      {
        "agenda_item_id": "92b0bc3b-7d14-4c46-96df-c01f35d9202a",
        "phrase": "review launch timeline"
      },
      {
        "agenda_item_id": "04515abc-1f84-4f53-af4d-5ef95a857b58",
        "phrase": "decide hiring priorities"
      },
      {
        "agenda_item_id": "81025425-2035-4058-8a65-dc2e0dc2e5bf",
        "phrase": "assign follow-up tasks"
      }
    ],
    "started_at": "2026-03-13T10:15:30Z",
    "ended_at": null
  },
  "router_rtp_capabilities": {},
  "existing_producer_ids": []
}
```

`meeting_joined` is sent only as the direct response to `join_meeting`.

Meeting snapshot event:

```json
{
  "type": "meeting_snapshot",
  "meeting": {
    "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
    "title": "Quarterly Product Review",
    "state": "active",
    "owner_participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
    "moderator_participant_id": "fa0d8ef6-c7f9-4d2c-a935-f4cc1e1cc5b0",
    "participants": [
      {
        "participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
        "kind": "human",
        "meeting_role": "owner",
        "name": "Frederic"
      },
      {
        "participant_id": "fa0d8ef6-c7f9-4d2c-a935-f4cc1e1cc5b0",
        "kind": "agent",
        "meeting_role": "moderator",
        "name": "Moderator"
      },
      {
        "participant_id": "5f3afccf-6dd6-486e-9a24-12e11384165f",
        "kind": "human",
        "meeting_role": "participant",
        "name": "Alice"
      },
      {
        "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
        "kind": "agent",
        "meeting_role": "participant",
        "name": "CTO"
      }
    ],
    "agenda": [
      {
        "agenda_item_id": "92b0bc3b-7d14-4c46-96df-c01f35d9202a",
        "phrase": "review launch timeline"
      },
      {
        "agenda_item_id": "04515abc-1f84-4f53-af4d-5ef95a857b58",
        "phrase": "decide hiring priorities"
      },
      {
        "agenda_item_id": "81025425-2035-4058-8a65-dc2e0dc2e5bf",
        "phrase": "assign follow-up tasks"
      }
    ],
    "started_at": "2026-03-13T10:15:30Z",
    "ended_at": null
  }
}
```

`meeting_snapshot` is sent without `request_id`.

- It must be sent to the joining participant immediately after `meeting_joined`.
- It may be re-sent later if meeting metadata changes.

Agent text event:

```json
{
  "type": "agent_text",
  "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
  "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
  "participant_name": "CTO",
  "utterance_id": "7c7ba7cf-4e00-441f-a653-00f1d4772fe4",
  "text": "We should hire infrastructure support before expanding the launch scope."
}
```

`agent_text` is broadcast to all connected clients in the meeting.

Agent audio event:

```json
{
  "type": "agent_audio",
  "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
  "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
  "participant_name": "CTO",
  "utterance_id": "7c7ba7cf-4e00-441f-a653-00f1d4772fe4",
  "mime_type": "audio/wav",
  "audio_base64": "<base64 wav bytes>"
}
```

`agent_audio` is broadcast to all connected clients in the meeting.

Meeting ended event:

```json
{
  "type": "meeting_ended",
  "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
  "ended_by_participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
  "ended_at": "2026-03-13T11:02:10Z"
}
```

If `meeting_ended` is sent as the direct response to `end_meeting`, it must echo the request `request_id` to the requester.

The same `meeting_ended` event must also be broadcast without `request_id` to all other connected clients in the meeting.

Minutes ready event:

```json
{
  "type": "minutes_ready",
  "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
  "minutes": {
    "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
    "title": "Quarterly Product Review",
    "owner_name": "Frederic",
    "moderator_name": "Moderator",
    "participants": [
      {
        "participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
        "kind": "human",
        "meeting_role": "owner",
        "name": "Frederic"
      },
      {
        "participant_id": "fa0d8ef6-c7f9-4d2c-a935-f4cc1e1cc5b0",
        "kind": "agent",
        "meeting_role": "moderator",
        "name": "Moderator"
      },
      {
        "participant_id": "5f3afccf-6dd6-486e-9a24-12e11384165f",
        "kind": "human",
        "meeting_role": "participant",
        "name": "Alice"
      },
      {
        "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
        "kind": "agent",
        "meeting_role": "participant",
        "name": "CTO"
      }
    ],
    "started_at": "2026-03-13T10:15:30Z",
    "ended_at": "2026-03-13T11:02:10Z",
    "agenda": [
      {
        "agenda_item_id": "92b0bc3b-7d14-4c46-96df-c01f35d9202a",
        "phrase": "review launch timeline",
        "decisions": [
          "The launch will remain on the current date."
        ]
      },
      {
        "agenda_item_id": "04515abc-1f84-4f53-af4d-5ef95a857b58",
        "phrase": "decide hiring priorities",
        "decisions": []
      },
      {
        "agenda_item_id": "81025425-2035-4058-8a65-dc2e0dc2e5bf",
        "phrase": "assign follow-up tasks",
        "decisions": [
          "Alice will draft the follow-up staffing proposal."
        ]
      }
    ]
  }
}
```

`minutes_ready` is broadcast to all connected clients in the meeting.

### Meeting Event Log Shape

Each recorded event in the meeting event log has this envelope:

```json
{
  "event_id": "d6762df8-a122-48e7-bf95-e56207d76852",
  "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
  "sequence_number": 1,
  "recorded_at": "2026-03-13T10:15:30Z",
  "event": {}
}
```

`event` must be one of:

```json
{
  "type": "meeting_created",
  "meeting": {}
}
```

```json
{
  "type": "human_utterance_recorded",
  "utterance_id": "a7de08e2-f47d-40e2-9b6f-651c4e05bd95",
  "participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
  "text": "CTO, what is the biggest risk in this plan?",
  "directly_addressed_agent_participant_ids": [
    "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2"
  ]
}
```

```json
{
  "type": "agent_floor_requested",
  "floor_request_id": "02f48207-5d7d-4ed8-92cf-82c2ebaf38f9",
  "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
  "source_utterance_id": "a7de08e2-f47d-40e2-9b6f-651c4e05bd95"
}
```

```json
{
  "type": "agent_floor_decision",
  "floor_request_id": "02f48207-5d7d-4ed8-92cf-82c2ebaf38f9",
  "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
  "granted": true
}
```

```json
{
  "type": "agent_turn_selected",
  "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
  "source_utterance_id": "a7de08e2-f47d-40e2-9b6f-651c4e05bd95",
  "reason": "direct_address"
}
```

```json
{
  "type": "agent_utterance_recorded",
  "utterance_id": "7c7ba7cf-4e00-441f-a653-00f1d4772fe4",
  "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
  "text": "The biggest risk is infrastructure load during onboarding.",
  "source_utterance_id": "a7de08e2-f47d-40e2-9b6f-651c4e05bd95"
}
```

```json
{
  "type": "meeting_ended",
  "ended_by_participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be"
}
```

```json
{
  "type": "minutes_generated",
  "minutes": {
    "meeting_id": "7b0609b6-53d4-4dbe-913e-4f2ea64b3f87",
    "title": "Quarterly Product Review",
    "owner_name": "Frederic",
    "moderator_name": "Moderator",
    "participants": [
      {
        "participant_id": "e60d2f9a-4d12-4f06-b5e7-6bd4a15991be",
        "kind": "human",
        "meeting_role": "owner",
        "name": "Frederic"
      },
      {
        "participant_id": "fa0d8ef6-c7f9-4d2c-a935-f4cc1e1cc5b0",
        "kind": "agent",
        "meeting_role": "moderator",
        "name": "Moderator"
      },
      {
        "participant_id": "5f3afccf-6dd6-486e-9a24-12e11384165f",
        "kind": "human",
        "meeting_role": "participant",
        "name": "Alice"
      },
      {
        "participant_id": "7f303650-577f-43a5-b0c8-e0f9b2d2d7a2",
        "kind": "agent",
        "meeting_role": "participant",
        "name": "CTO"
      }
    ],
    "started_at": "2026-03-13T10:15:30Z",
    "ended_at": "2026-03-13T11:02:10Z",
    "agenda": [
      {
        "agenda_item_id": "92b0bc3b-7d14-4c46-96df-c01f35d9202a",
        "phrase": "review launch timeline",
        "decisions": [
          "The launch will remain on the current date."
        ]
      },
      {
        "agenda_item_id": "04515abc-1f84-4f53-af4d-5ef95a857b58",
        "phrase": "decide hiring priorities",
        "decisions": []
      },
      {
        "agenda_item_id": "81025425-2035-4058-8a65-dc2e0dc2e5bf",
        "phrase": "assign follow-up tasks",
        "decisions": [
          "Alice will draft the follow-up staffing proposal."
        ]
      }
    ]
  }
}
```

## Persistence and Source of Truth

The meeting utterance stream is the shared semantic input for all participants.

- Finalized human utterances enter the stream through speech-to-text.
- Agent utterances enter the stream from generated text before TTS.
- Agents observe utterance events, not synthesized audio.
- TTS is for human playback only.

This means agents "hear" each other through the shared utterance stream, not through room audio playback.

## Output Artifacts

### Minutes

The minutes are a written document generated when the meeting ends.

- Only the moderator generates the minutes.
- The minutes are not spoken aloud.

The minutes document must contain:

- meeting title
- owner
- moderator
- participants
- started at
- ended at
- agenda

For each agenda item, the minutes must include:

- `phrase`
- `decisions`

`decisions` is a list. It may be empty.

The agenda in the minutes is the original agenda annotated with the decisions made for each item.

Every original agenda item must appear in the minutes in the original order.

## Meeting End

- Only the owner may end the meeting.
- Ending the meeting is an explicit meeting event.
- After the meeting-end event, no further speaking turns are accepted.
- After the meeting-end event, the moderator generates the minutes from the agenda and meeting event log.

## Acceptance Criteria

The feature is complete when all of the following are true:

- a meeting can be created with one owner, one implicit moderator, an initial participant list, and an agenda
- finalized human utterances are appended to the meeting utterance stream
- non-moderator agents can speak only by direct address or moderator-granted floor request
- the moderator can choose at most one agent to speak at a time
- generated agent text is appended to the meeting utterance stream
- generated agent audio is delivered to clients as a local playback clip, not SFU media
- agents observe one another through the shared utterance stream
- only the owner can end the meeting
- ending the meeting triggers moderator-generated written minutes from the fixed template

## Open Questions
