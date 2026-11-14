# Experimental Delta Chat calls

Sigma can answer **audio calls to native Codex bridges**, restricted to the bot master. Native Grok Build and custom text adapters are not advertised as call-capable. Unrecorded incoming calls require no browser or additional WebRTC/codec library. Opt-in outgoing calls use installed Chromium as described below; no Cargo dependency or browser package is installed by Sigma.

## Use it

Call the Codex bot using Delta Chat's audio-call button in your private bot-master conversation. The bot forwards your SDP offer to the installed Codex app-server's experimental `thread/realtime/start` WebRTC transport and returns `thread/realtime/sdp` through chatmail core's `accept_incoming_call`. Delta Chat includes a video transceiver in audio-only offers. Sigma offers only the bundled audio section to the provider, preserves its mid and bundled ICE candidates, and restores rejected video and application sections in the answer to keep Delta Chat’s media ordering. Delta Chat’s negotiated SCTP channels carry mute/ICE messages that are incompatible with provider realtime events and can close the provider transport; they must not be forwarded. Codex uses its server-side connection for realtime controls instead. Initial SDP candidates must establish the audio connection; Delta Chat’s data-channel ICE trickling and remote mute indicators are unavailable. Actual video-call attempts remain declined. Audio is exchanged directly between the calling client and OpenAI's realtime service; Sigma does not decode, transcribe or re-encode audio in this default direct mode. With `--record-calls`, the local browser bridge handles both sides and records as described below.

- After acceptance, Sigma requests a brief spoken greeting.
- `/call` shows this bridge's call capability and limitations.
- `/call stop` ends the current call. Delta Chat's hang-up button works too.
- Only the current bot master may call, even if another person has an approved text/voice-note permission. Other callers are declined without starting a provider session.
- Only one call runs per bridge. Video calls are declined with a request to use audio. Typed administration commands remain available during a call; ordinary text turns are not run concurrently against an incoming call’s same session. During outgoing calls, the master’s private chat remains available because the voice call has its own session.
- A master transfer invalidates the call's scope. The call checks that scope every second. The bridge also ends calls on remote hang-up, provider failure, shutdown, or a 20-minute limit. Negotiation is bounded to 60 seconds.

The backing Codex thread uses the selected text model and its saved thread ID. Resume requests use `excludeTurns: true` so large conversation histories are kept inside Codex rather than copied into the signaling response. This preserves the thread’s history and avoids the 1 MiB signaling-event limit. The realtime voice service selects its own configured realtime model; Sigma does not pass a text-model ID as an audio-model ID. Master-private sessions stay separate after a transfer. The scoped chat tools are available during a call; client-side requests to approve permissions or execute external tools are refused. Spoken administration commands do not change mastery, access or model settings; type those commands instead.

## Research and verification

Research performed on 2026-11-13 with installed `codex-cli 0.160.0`, `grok 1.0.77` and the pinned chatmail core. The installed Codex schema generator reports the experimental WebRTC transport. A live negotiation probe using a Delta-style offer (including its unused video transceiver) obtained a valid SDP answer using the existing login, both directly and through Sigma's isolated app-server path. A subsequent headless Chromium media test reproduced a disconnect when Delta Chat’s mute/ICE messages were sent. With those channels rejected, WebRTC remained connected for 36 seconds and received non-silent speech (verified through inbound RTP packets and audio energy). The test used the existing installed Playwright/Chromium tooling; no project dependency was added. **Phone-to-agent audio, connectivity across NATs and interruption behavior still require a successful real Delta Chat call test.**

Primary sources:

- [Codex app-server documentation](https://learn.chatgpt.com/docs/app-server) and the [upstream realtime protocol](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/v2/realtime.rs): thread-scoped realtime, WebRTC SDP offers/answers, sideband events. The installed CLI's generated schema is the implementation reference.
- [Delta Chat calls webapp](https://github.com/deltachat/calls-webapp): SDP signaling through core, browser-managed WebRTC, ICE gathering and negotiated channels. Core's existing `calls` module supplies incoming/ended events and accept/end operations.
- [Grok CLI reference](https://docs.x.ai/build/cli/reference) and installed CLI help: no verified live-call transport for Grok Build. xAI separately documents a [speech-to-speech API](https://docs.x.ai/developers/model-capabilities/audio/speech-to-speech) and a [WebRTC cookbook](https://docs.x.ai/cookbook/examples/voice-agent-webrtc) requiring API credentials and a separate media bridge. That is not interchangeable with the installed CLI subscription/session, so Sigma does not claim or fake Grok call support.

Regression checks cover event-only outgoing acceptance (including ignoring another call’s answer), authenticated master cancellation, denial for guests/groups, and idempotent retries. A live Codex test of “Hey, cancel the call please” verified the queued request was actually canceled. A separate live realtime audio probe received non-silent Codex speech through the outbound browser bridge. These checks do not replace a phone-to-agent call test.

Provider account access, live-audio quotas and experimental protocol versions can affect availability. A provider error declines/ends the call with a short message to the still-authorized master. Offers/answers are not printed in Sigma logs. `tests/fixtures/call-offer.sdp` is a negotiation-only offer with a dummy fingerprint, not a real media connection.

## Opt-in outbound calls

Start a **native Codex** bridge with `--allow-outbound-calls` to let it place Delta Chat audio calls at its bot master's request:

```sh
./sigma agent-connect codex --to me --alias assistant --model gpt-6.1-sol --allow-outbound-calls
# Existing identity (stop its worker first):
./sigma agent-connect codex --agent-id 1 --account codex-new --allow-outbound-calls
```

This permission is off by default and applies only to this worker's lifetime. Include the flag on each restart; restarting without it disables outgoing calls. `agent-list --json` reports `outbound_calls`. Grok and custom adapters reject the flag because their outbound audio transport is not implemented.

In the master's private chat, share one contact card and say “Call this person and ask when tomorrow's meeting starts.” Alternatively, supply a contact ID, saved contact alias, address, or Delta Chat contact invitation link. Calling another agent uses that agent's Delta Chat contact. Its receiving policy still applies: a bot that only accepts calls from its own master may decline.

The agent receives this authenticated tool in its instructions. Resumed Codex turns also receive the current capability catalog in the turn input, because `exec resume` can retain the original developer instructions. This keeps existing conversations aware of newly enabled or disabled tools without resetting their history:

```sh
"$SIGMA_BIN" agent-act call TARGET "What to say or accomplish"
"$SIGMA_BIN" agent-act call message:123 "Ask when tomorrow's meeting starts"
"$SIGMA_BIN" agent-act call-cancel all
```

`message:123` must refer to a single contact card sent by the master in this private chat. The trusted worker imports the original attachment; an agent-edited copy is not used. The tool runs inside the agent's capability scope, not as an unrestricted terminal dialer. Call actions cannot be batched. A successful result means **queued**, not answered; the worker reports dialing and termination in the master's chat. A call starts after the current text turn ends. There is one call at a time, a 90-second ringing limit and a 20-minute overall limit. `/call stop` cancels setup, ringing or an active call. Master transfer or lost authorization ends it. Calls never grant the recipient text access or mastery.

After the recipient answers, Sigma consumes core’s `OutgoingCallAccepted` event (the remote answer SDP is not persisted in the call record), connects the local audio bridge, and starts Codex realtime voice. Calls use actual realtime audio, not a text tool or a simulated voice response.

The master can type “cancel the call” during an outgoing call. The agent has `agent-act call-cancel all`, or a specific call ID/request UUID, and must report the tool result. This authenticated action is restricted to the current master’s private chat, works without outbound dialing permission, and cannot be batched. `all` stops this agent’s queued requests and open calls in its account. Guests and group chats cannot use it. `/call stop` remains an immediate deterministic command during either incoming or outgoing calls.

Each outgoing call uses a fresh isolated Codex conversation with only the supplied call brief. It does not inherit the master's private chat history or scoped messaging/administration credentials. Do not put private information in the brief unless it is intended for the recipient. The caller identifies itself as an AI assistant. The call does not generate a transcript report in the master's chat.

### Optional browser runtime

Codex's installed WebRTC interface answers offers; it does not originate them. Outbound calls therefore use **an installed Chromium** as a local audio bridge. Sigma controls it over private inherited pipes, with a fresh browser profile and no debugging TCP port. Chromium handles WebRTC, codecs and audio forwarding between the recipient and Codex; Delta Chat's mute/ICE channels terminate locally. Outbound audio passes through this machine. Incoming calls retain the existing direct provider path.

No Cargo dependency or browser package is installed by Sigma. It checks `SIGMA_CALL_BROWSER` (absolute executable path), standard Chromium/Chrome executables on `PATH`, then an existing Playwright browser cache. To choose a browser explicitly:

```sh
SIGMA_CALL_BROWSER=/absolute/path/to/chromium ./sigma agent-connect codex --to me --allow-outbound-calls
```

Keep the installed browser current. Chromium runs with its sandbox enabled. TURN/STUN configuration comes from chatmail core. If Chromium is absent or cannot run, outgoing calls fail with an explanation. The flag is not available for browsers on remote machines.

## Call history and cancellation from the terminal

```sh
./sigma agent-calls codex --account codex-new
./sigma agent-calls codex 100 --account codex-new --json
./sigma agent-call-cancel codex 123 --account codex-new
./sigma agent-call-cancel codex REQUEST_UUID --account codex-new
./sigma agent-call-cancel codex all --account codex-new
```

`agent-calls ID_OR_ALIAS [COUNT]` lists the newest 50 entries by default (maximum 100). It combines the selected account's stored incoming/outgoing calls, including archived chats, with this agent's outbound request journal. Actual calls include numeric `call_id`, chat, direction, core `call_status`, timestamp and connected `duration_seconds`. Outbound requests also have a UUID `request_id`, request `status`, and a failure reason when applicable. A request can fail before any call message exists; it will still appear in the journal. Queued and setup time are not counted as connected duration. Core call history predates this feature; request-only failure history starts with this version.

Cancel by numeric call ID after dialing starts, or by request UUID during queue/setup. `all` cancels this agent's queued requests and all currently open calls in the selected account. Commands work while the worker is running through its control socket, or while stopped through core. They require the trusted terminal and are unavailable inside agent turns. In chat, the master can use `/call stop` or ask the agent to use its separately authenticated `agent-act call-cancel` tool. Canceling does not delete history. Interrupted queued requests are not redialed after a restart.

The journal is stored privately under `agents/ALIAS/calls/`. It contains status, identifiers, timestamps and bounded error text; call briefs, invitation secrets and audio are not saved there. Sigma continues to use core APIs for all call database reads and updates.

Shared vCards may include large embedded profile photos. Outbound calling accepts cards up to 8 MiB, removes the photo (including folded lines) before importing identity/key fields, and keeps a 64 KiB bound on the remaining identity data. The original shared attachment is unchanged.

### Diagnosing silent outgoing calls

`agent-calls` shows `stage`; its JSON output also includes the latest `media` snapshot (audio packet counts, ICE/connection states and AudioContext state), updated every five seconds during voice setup and the call. It records no audio, transcript, SDP, network addresses or credentials. `waiting-for-answer` means Sigma has not received the recipient client’s answer; Codex voice cannot start yet. Delta Chat’s client can display “in-call” when it receives a media track, before it has finished creating/sending its answer. `answer-received`, `starting-codex-voice`, `connecting-audio` and `audio-connected` distinguish later stages; connected transports alone do not prove audible speech.

The outgoing bridge uses Delta Chat’s `max-bundle` policy so audio and control channels share one ICE transport. Initial candidate gathering waits up to ten seconds per leg before using the candidate snapshot and subsequent data-channel trickling. A live test using Delta Chat’s actual CallsManager code and Codex realtime received non-silent speech locally. Phone/network-specific failures still need a real call attempt and the resulting stage diagnostics; local success does not establish phone audibility.

### Call purpose and agent hang-up

Outgoing voice sessions receive an initial developer message identifying the conversation as a live outgoing Delta Chat call, naming the master and recipient from their contact labels, and including the master-authorized brief. This context is also repeated when audio connects. The assistant introduces itself as an AI calling on the master's behalf and states the purpose, even if the recipient opens with “how are you?”. Incoming voice sessions also receive their call context explicitly at startup.

The outgoing call's isolated backing Codex thread exposes one dynamic tool, `sigma_hang_up`, with empty arguments. It ends only that worker's current call. Sigma validates the thread, tool name, arguments and current master authorization before ending the core call. It cannot select other calls or administer the account. The agent may end a naturally finished conversation or comply with the callee's request to stop; it should say goodbye first.

Realtime speech does not reliably invoke backing tools on its own. Sigma therefore sends completed assistant utterances to the isolated backing agent for a short call-lifecycle decision. The agent either keeps the call or invokes the hang-up tool; there is no keyword-triggered hang-up. Control responses are kept out of the voice stream with client-managed handoffs. These transient call utterances do not enter the master's private thread or the call-history journal. Hang-up is recorded as `last_stage: agent-hung-up` and normal completion. The master-private text chat retains its authenticated `agent-act call-cancel` tool and `/call stop` command.

Validation: a live Codex realtime test responded to “Hello, how are you?” by introducing itself on the bot master's behalf and delivering the “you are amazing” brief; after a goodbye it issued `sigma_hang_up`. Offline tests reject other-thread, other-tool and recipient/call-ID override requests. Tool integration follows the [Codex app-server application-owned tools model](https://developers.openai.com/blog/codex-as-a-platform), using the installed CLI's experimental schema.

## Record, keep and send calls

Recording is opt-in for native Codex bridges:

```sh
./sigma agent-connect codex --agent-id 1 --account codex-new --allow-outbound-calls --record-calls
./sigma agent-calls codex --account codex-new --json
```

Include `--record-calls` on each restart to record both incoming and outgoing audio calls. Without it, recording is disabled. `agent-list --json` reports `record_calls`; `/call` reports the setting too. The agent is instructed to announce recording in its opening sentence. Existing Chromium is required; no new dependency is installed. Recorded incoming calls use the local Chromium bridge instead of the direct client-to-provider path.

The browser mixes both sides into an Opus WebM audio file. Sigma streams chunks to a private folder during the call and finalizes on remote hang-up, agent hang-up, cancellation, normal worker shutdown, or a handled call error. The completed recording is sent as a file with the call ID to the current authorized bot master's private chat, not to the callee/group. Master transfer revokes delivery to the old master; the private local copy remains. A queued attachment is not a delivery receipt. If sending fails, the saved copy remains for manual sending.

Files are kept under `ACCOUNT_ROOT/agents/ALIAS/recordings/call-CALL_ID.webm`, with a small `.json` record describing status and the queued message ID. On this installation: `~/.config/sigma-accounts/accounts/codex-new/agents/codex/recordings/`. The directory is mode 0700 and files are mode 0600 on Unix, outside the agent's writable outbox. `agent-calls` displays the path; JSON also exposes the recording status. Recordings are retained until you delete them or delete the account. Sending a recording creates a separate attachment copy in the account.

Recording is bounded to 32 MiB and the existing 20-minute call limit. Files ending in `.webm.part` are in progress, or incomplete after an abrupt process crash/power loss; their last buffered audio may be missing. A handled recorder failure preserves already-written audio and labels the sent file partial where possible. No recordings are created retroactively for old calls. This records audio only, not video or transcripts.

### Audio-only client compatibility

The Delta-facing outgoing offer follows Delta Chat's own ordering: video, then audio, plus its control channels. A tiny 16×16 black-frame track at one frame per second accompanies audio in the same MediaStream. This matches the muted-camera behavior of normal Delta Chat calls and avoids leaving video-based recipient players waiting for a first frame. No camera is accessed, video remains reported as disabled, and the provider-facing connection remains audio-only. Received video tracks are ignored by the audio mixer. This matters for recorded incoming calls too: Delta Chat offers a video transceiver even for audio-only calls. Treating its track as a Web Audio source raises an error and can terminate an otherwise connected call.

Media diagnostics also expose negotiated audio directions, outgoing source energy when available, and remote RTCP report/loss counts. Packet transmission or a non-silent local recording alone does not establish playback on the recipient's device. A live test with the receiving client in a separate browser process received Codex speech; Android-device playback remains a separate validation step.

The outgoing compatibility test verifies both audio and black-frame tracks arrive in one remote stream and that the recipient decodes video frames, while receiving non-silent audio. A live Codex test also exercises the receiving client's hidden video/audio player in a separate browser process. These tests do not substitute for checking the recipient's Android device. `compatibilityFramesSent` in call diagnostics shows whether the local sender encoded the placeholder frames. Recordings still contain audio only.
