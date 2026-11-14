use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::{attachment_message, text_file_message};
use crate::shared::{ensure_configured, send_to_target};
use deltachat::context::Context;
use deltachat::message::Viewtype;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "send-file",
    arguments: "TARGET FILE",
    description: "Send a local file to a contact ID, saved alias, or full contact invitation; use --chat for an existing chat/group/channel ID. Failed submissions may remain queued; use listen to retry.",
    options: "  --chat             TARGET is a numeric existing chat/group/channel ID\n  --caption TEXT     Optional attachment caption (incompatible with --as-text)\n  --as-text          Send UTF-8 file contents as one text message; reject empty/invalid text\n  --timeout SECONDS  Secure-join wait limit (default: 120)\n",
    example: "send-file 10 ./message.txt --as-text",
    summary: "Send a local file to a contact ID, alias, or invitation",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let message = if args.option("--as-text").is_some() {
        text_file_message(std::path::Path::new(&args.positional[1]))?
    } else {
        attachment_message(
            context,
            std::path::Path::new(&args.positional[1]),
            args.option("--caption"),
            Viewtype::File,
        )?
    };
    send_to_target(context, args, message).await?;
    Ok(())
}
