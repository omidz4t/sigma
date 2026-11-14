export const agentSlides = [
  {
    title: 'Your agent. In your chat. On a call.',
    description: 'Connect Codex or Grok to Delta Chat. Keep the conversation going from your phone, and let Codex make a call when you ask.',
  },
  {
    title: 'Your own research team.',
    description: 'Meet Atlas, your Codex researcher, and Nova, your Grok researcher. Give each a dedicated account, a custom profile photo, and a focused starting prompt. Bring them into your research chat.',
    command: './sigma agent-connect codex --new-account atlas \\\n  --to "YOUR_INVITATION" --alias atlas --name "Atlas" \\\n  --profile-photo ./atlas.png\n./sigma agent-connect grok --new-account nova \\\n  --to "YOUR_INVITATION" --alias nova --name "Nova" \\\n  --profile-photo ./nova.png',
    note: 'The first --to recipient is the bot master. Add the agents to a group; they respond only to their master and approved people. Shown here: real Atlas and Nova agents in an isolated Delta Chat demo group.',
    visual: 'chat',
  },
  {
    title: 'A name. A face. Your own agents.',
    description: 'Atlas and Nova wear Sigma’s Moai look, with an explorer hat and a navy cap. Their own accounts keep each identity distinct, together in a research group with its own team photo.',
    note: 'Real Delta Chat group profile. The bot master controls who can talk to each agent.',
    visual: 'profiles',
  },
  {
    title: 'Send audio. Know what is supported.',
    description: 'Voice notes and audio files reach the agent in the same conversation. It transcribes using its own tools when available. This demo received a ❌ because its Codex CLI could not transcribe the recording.',
    note: 'Actual audio test, not a simulated transcript. Support depends on the provider, model, and installed tools.',
    visual: 'voice',
  },
  {
    title: 'The result arrives as a file.',
    description: 'Ask for a research checklist and receive the actual attachment in Delta Chat. Open it, save it, or continue the conversation with your agent.',
    note: 'Atlas generated and sent this text file in response to a typed request. It was not produced from the unsupported audio recording.',
    visual: 'file',
  },
  {
    title: 'Give it permission to call.',
    description: 'Outgoing calls are an explicit choice. Start Codex with --allow-outbound-calls, then give it a contact and a clear reason to call.',
    command: './sigma agent-connect codex --to me --alias caller \\\n  --allow-outbound-calls',
    note: 'Experimental Codex audio calls use your installed Codex login and Chromium. Grok supports agent chat; live Grok calls are not implemented.',
    visual: 'permission',
  },
  {
    title: 'Share a contact. Say what to say.',
    description: 'Share a Delta Chat contact card and tell the agent what to say. In this controlled demo, Bot Master shares their own card and asks Atlas to call with a message of appreciation.',
    note: 'Sigma resolves the original contact card and queues a call. Queued means requested; the recipient still needs to answer.',
    visual: 'contact',
  },
  {
    title: 'Answer. Talk to your agent.',
    description: 'The demo owner answers Atlas’s outgoing call in Delta Chat. Codex starts a realtime voice session using the supplied call brief.',
    note: 'The call gets the brief, not your private chat history. You can type “cancel the call” or /call stop in the master chat, even while it is ringing.',
    command: '/call stop',
    visual: 'call',
  },
  {
    title: 'You stay in control.',
    description: 'Check call history from the terminal. Enable recording only when you want it: completed recordings stay locally and are sent back to the bot master.',
    command: './sigma agent-calls caller\n./sigma agent-call-cancel caller all\n./sigma agent-list\n# Add --record-calls when starting the Codex agent.',
    note: 'Only the bot master can authorize outgoing calls. Sharing a contact does not give that person access to your agent. Recording is announced at the start of the call.',
  },
];
