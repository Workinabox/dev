# Feature 1

## Current Starting Point

The current system supports:

- joining a room from the mobile app
- sending audio to the backend
- transcribing that audio on the backend
- logging the resulting transcript text to the server console

At this stage, the transcript is not yet sent to an LLM, no assistant response is generated, and no synthesized speech is sent back to the app.

## Desired Next Step

The next feature is a conversational assistant loop:

1. user speaks in a room
2. backend transcribes the utterance
3. backend sends the transcribed text to an LLM
4. LLM produces a text response
5. backend converts that response to speech using a TTS model or service
6. backend sends the assistant response back to the app as text and audio

## First Implementation Goal

The first version should be simple and robust rather than fully real-time.

Recommended first shape:

- only trigger the assistant on finalized utterances, not partial transcript chunks
- keep conversation state on the backend
- avoid overlapping assistant replies
- prevent the assistant from responding to its own synthesized speech

The backend should stop treating transcription as console-only output and instead emit finalized utterance events such as:

- `room_id`
- `peer_id`
- `text`

Those events can then feed the assistant pipeline.

## TTS Clarification

The current Whisper model is speech-to-text only. It cannot perform text-to-speech.

That means the full assistant loop requires three distinct capabilities:

- speech-to-text
- text generation
- text-to-speech

Whisper continues to cover only the first of those.

## Local-First Path

For the temporary local version, the LLM and TTS can run on the same machine as the backend.

The preference is to keep that inside the current backend process as much as possible, similar to the current Whisper arrangement:

- Rust backend process
- Rust bindings
- native inference implementation underneath is acceptable

The strongest candidate discussed for local in-process LLM inference is an embedded `llama.cpp` path through Rust bindings. The important clarification is that `llama.cpp` does not require a separate server process. It can also be embedded as a library and called from Rust.

For TTS, a separate local TTS model or engine will still be required.

## Long-Term Path

The local model path is only temporary.

The longer-term direction is to use remote model APIs for higher-end agents, for example hosted frontier models. In that final setup:

- LLM calls will mostly be network I/O
- those calls can be awaited directly in Tokio
- the backend should keep the agent orchestration logic independent from the specific provider

This implies a provider abstraction:

- local provider implementation for temporary in-process models
- remote provider implementation for future API-based models

The same principle applies to TTS.

## Tokio and Blocking Work

Tokio remains the right orchestration layer for the backend overall.

It should continue to manage:

- WebSocket signaling
- room coordination
- agent task orchestration
- network-based model and TTS calls

The caution is only about local inference work. In-process local LLM inference and local TTS synthesis are blocking, CPU-heavy, or native-library-heavy tasks. They should not run directly on Tokio executor threads.

For the local phase:

- Tokio should orchestrate the workflow
- local model and TTS work should run behind blocking worker boundaries
- results should be sent back into the async system after inference completes

This is the same basic pattern already used by the current local transcription worker.

## Concurrency Model

A future meeting may contain multiple agents, but that does not necessarily mean loading one totally separate model instance per agent.

The safer starting point is:

- one shared local model runtime
- a bounded number of concurrent inference workers
- per-agent conversation state
- queueing and backpressure

For the eventual remote API model path, one Tokio task per agent or session becomes much more natural because the work is primarily waiting on network responses rather than consuming CPU locally.

## Delivery Back to the App

The cleanest first version is not to inject assistant audio into the SFU immediately.

Instead, the backend can extend the existing signaling channel with assistant-specific events, for example:

- `assistant_text`
- `assistant_audio`

In this first version:

- assistant text is sent back over the signaling socket
- assistant audio is sent back as a clip or payload the app can play locally

This keeps the first implementation much simpler.

A later version can make the assistant behave like a synthetic room participant and inject assistant audio into the SFU as if it were another speaker.

## Recommended Architecture Direction

The architecture should be designed around an async backend boundary even if the temporary implementation is local.

That means the assistant orchestration logic should not care whether the underlying provider is:

- a local in-process model behind a blocking worker
- or a remote model API awaited directly through Tokio

This will allow the first local version to ship without forcing a rewrite when the system later moves to hosted models.
