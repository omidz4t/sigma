# Voice messages

Send an existing local recording as a voice message using chatmail core's `Voice` message type:

```sh
./sigma send-voice CONTACT_ID ./recording.ogg --duration 3500
./sigma send-voice CHAT_ID ./recording.ogg --chat --duration 3500
./sigma send-voice 'https://i.delta.chat/#FULL_INVITATION' ./recording.ogg
```

Contact IDs, secure-join invitations, and existing chat/group/channel IDs follow the same rules as `send-file` and `send-photo`. `--chat` selects a chat ID. Sending to incoming read-only channels is rejected through core. Account storage defaults to `~/.config/sigma`; override it with `--data-path`.

`--duration` is optional metadata in milliseconds, from 0 to 2147483647; 0 or omission means unknown. The CLI does not measure duration, record your microphone, transcribe, validate codecs, or convert audio. Supply an actual recording in a format supported by your recipients. Empty files are rejected. `--caption` optionally supplies message text, whose presentation depends on the receiving client. Use `send-file` for a regular file attachment.

```sh
./sigma send-voice 10 ./recording.ogg --duration 3500 --caption 'Voice note' --json
./sigma send-voice --help
```

The JSON send result contains `message_id` and `chat_id`. `read-messages --json` and `listen --json` show `viewtype: "Voice"`, attachment details and `duration_ms`. A successful send means SMTP submission; playback needs to be checked in the receiving app. Failed submissions may remain queued; run `listen` before retrying.

`tests/fixtures/voice-tone.wav` is a one-second synthetic test tone, not a speech recording. It is used to verify voice-message type, unchanged audio bytes, and duration metadata through core without adding audio dependencies.

## Incoming voice notes for agents

Authorized voice notes and audio attachments reach the same persistent provider conversation as typed messages. Sigma stages the original recording in that chat's private outbox and gives its path/type to the agent. It does not run speech recognition or create a transcript itself.

The agent must transcribe the recording using its own available audio/transcription capabilities, retain that transcription in its conversation and then answer the spoken request. A separate transcript message is only sent if requested. It must not infer speech from filenames, captions or duration metadata. Support depends on the active provider CLI, model and available tools; a model's general audio support does not guarantee that its CLI can process a local recording.

When the agent cannot transcribe or obtain a usable transcript, it returns exactly `[SIGMA_VOICE_UNSUPPORTED]`. Sigma consumes that result and queues a ❌ reaction on the original voice/audio message, suppressing its normal text reply. If core refuses the reaction, Sigma tries a short quoted “Voice transcription is not supported or is unavailable for this recording” message instead. A missing recording also takes this unsupported path. Provider execution/network errors remain ordinary bridge errors rather than proof of unsupported audio.

Native starting instructions and `SIGMA_AGENT_INSTRUCTIONS` describe this workflow. Custom adapters receive the same audio task on stdin and the staged path in `SIGMA_INCOMING_VOICE_PATH`, plus optional `SIGMA_INCOMING_VOICE_DURATION_MS` metadata. Adapters perform their own transcription and return their response as usual, or the unsupported marker; Sigma never invokes an external transcription service on their behalf. Existing `[SIGMA_NO_REPLY]` suppression remains available after the agent sends its own messages through tools.

Sender permissions are checked before staging/invocation, and checked again before queuing the unsupported reaction or an automatic reply. Unapproved people do not get voice responses or reactions. Spoken `/approve`, `/access` or `/model` instructions are not owner administration commands; send those as typed chat commands. Group recordings and replies follow the existing group permission and conversation boundaries.
