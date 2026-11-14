use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "agent-act",
    arguments: "ACTION ARGS...",
    description: "With an explicitly enabled native Codex bridge, call TARGET BRIEF queues an outbound audio call from the bot master’s private chat. Send token-authorized actions to the current agent's fixed chat/account while its worker is online. Requires SIGMA_AGENT_SOCKET and SIGMA_AGENT_CAPABILITY supplied to that agent. Actions: messages [COUNT [BEFORE_MESSAGE_ID]]; send-webxdc PATH; webxdc-update MESSAGE_ID JSON; webxdc-updates MESSAGE_ID [AFTER_SERIAL]; webxdc-realtime MESSAGE_ID TEXT; send-message TEXT; reply MESSAGE_ID TEXT; send-voice/file/video/photo PATH; react MESSAGE_ID [EMOJI]; edit-message MESSAGE_ID TEXT; set-name TEXT; set-photo PATH; batch JSON_FILE. Media paths must be inside SIGMA_AGENT_OUTBOX. No recipient or account overrides. Results are JSON and report queued IDs; batches may partially complete on runtime errors.",
    options: "  --caption TEXT    File/photo/video/voice caption\n  --duration MS     Voice/video duration in milliseconds\n  --request-id UUID  Reuse the same ID when retrying identical actions\n",
    example: "agent-act send-photo image.png --caption \"Hello\"",
    summary: "Use an agent's private, bound-chat messaging tools",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    crate::agent_actions::client(args).await
}
