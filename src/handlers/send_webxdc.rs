use crate::{
    Result,
    cli::Args,
    help::CommandHelp,
    shared::{attachment_message, ensure_configured, send_to_target_id},
};
use deltachat::{
    context::Context,
    message::{Message, Viewtype},
};
use std::path::Path;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "send-webxdc",
    arguments: "TARGET APP.xdc",
    description: "Send a local webxdc app to a contact ID, saved alias, or full contact invitation. Use --chat to interpret TARGET as an existing chat/group/channel ID. Core validates the package and enforces chat permissions. Failed submissions may remain queued; use listen before retrying.",
    options: "  --app NAME         Save this app instance under a durable name for app-* commands\n  --chat             TARGET is a numeric chat ID, including groups\n  --caption TEXT     Optional app message caption\n  --timeout SECONDS  Secure-join wait limit (default: 120)\n",
    example: "send-webxdc 12 ./app.xdc --chat --caption \"Try this app\"",
    summary: "Send a webxdc app to a contact or chat",
};

pub(crate) async fn handle(context: &Context, args: &Args, root: &Path) -> Result<()> {
    ensure_configured(context).await?;
    if let Some(name) = args.option("--app") {
        crate::apps::preflight(root, name)?;
    }
    let message = prepare(
        context,
        Path::new(&args.positional[1]),
        args.option("--caption"),
    )
    .await?;
    let id = send_to_target_id(context, args, message).await?;
    if let Some(name) = args.option("--app") {
        crate::apps::bind(context, root, name, id).await?;
    }
    Ok(())
}

pub(crate) async fn prepare(
    context: &Context,
    path: &Path,
    caption: Option<&str>,
) -> Result<Message> {
    if !path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("xdc"))
    {
        return Err("webxdc app must have a .xdc extension".into());
    }
    let message = attachment_message(context, path, caption, Viewtype::Webxdc)?;
    // Use core's ZIP reader to reject corrupt archives and missing entry points
    // before starting a secure-join handshake or queuing an outgoing message.
    message
        .get_webxdc_blob(context, "index.html")
        .await
        .map_err(|error| format!("invalid webxdc package (requires index.html): {error}"))?;
    Ok(message)
}
