use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::{ensure_configured, send_to_target};
use deltachat::context::Context;
use deltachat::message::Message;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "send-message",
    arguments: "TARGET TEXT",
    description: "Send quoted text to a numeric contact ID, saved alias, or full secure-join contact invitation; use --chat for an existing chat/group/channel ID. Wait for secure join before submitting. SMTP success does not guarantee recipient delivery.",
    options: "  --chat             TARGET is a numeric existing chat/group/channel ID\n  --timeout SECONDS  Wait limit in seconds (default: 120)\n",
    example: "send-message 10 \"Hello!\"",
    summary: "Send to a contact ID, alias, or secure-join invitation",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let message = Message::new_text(args.positional[1].clone());
    send_to_target(context, args, message).await?;
    Ok(())
}
