use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::attachment_message;
use crate::shared::{ensure_configured, send_to_target};
use deltachat::context::Context;
use deltachat::message::Viewtype;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "send-photo",
    arguments: "TARGET PHOTO",
    description: "Send a local image as a photo to a contact ID, saved alias, or full contact invitation; use --chat for an existing chat/group/channel ID. Core processes the image; failed submissions may remain queued.",
    options: "  --chat             TARGET is a numeric existing chat/group/channel ID\n  --caption TEXT     Optional photo caption\n  --timeout SECONDS  Secure-join wait limit (default: 120)\n",
    example: "send-photo 10 ./photo.jpg --caption \"Hello!\"",
    summary: "Send an image as a photo message",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    ensure_configured(context).await?;
    let message = attachment_message(
        context,
        std::path::Path::new(&args.positional[1]),
        args.option("--caption"),
        Viewtype::Image,
    )?;
    send_to_target(context, args, message).await?;
    Ok(())
}
