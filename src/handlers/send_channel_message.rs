use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::{ensure_configured, outgoing_channel};
use deltachat::context::Context;
use deltachat::{chat, message::Message};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "send-channel-message",
    arguments: "CHANNEL_ID TEXT",
    description: "Post quoted text to a broadcast channel owned by this account. Failed submissions may remain queued; use listen before retrying.",
    options: "",
    example: "send-channel-message 12 \"Hello subscribers!\"",
    summary: "Post to your broadcast channel",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let channel = outgoing_channel(context, args.positional[0].parse()?).await?;
    let mut message = Message::new_text(args.positional[1].clone());
    let id = chat::send_msg_sync(context, channel, &mut message).await?;
    crate::output::record(
        "sent",
        vec![
            ("message_id", id.to_u32().into()),
            ("chat_id", channel.to_u32().into()),
        ],
        &format!("Sent channel message {}", id.to_u32()),
    );
    Ok(())
}
