use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "send-webxdc-realtime",
    arguments: "MESSAGE_ID DATA",
    description: "Advertise/join an app's realtime channel, wait for a peer, send UTF-8 DATA, then leave after a short outgoing-queue grace period. With --file, DATA is a binary file path. Up to 128000 bytes. Both peers must be online in the same app instance. This is best effort, without delivery acknowledgement or saved history. Use listen-webxdc-realtime --send for a session that also receives replies.",
    options: "  --file             Read DATA from a binary file\n  --timeout SECONDS  Connection wait limit (default: 120)\n",
    example: "send-webxdc-realtime 83 'Hello live!' --timeout 60",
    summary: "Send ephemeral text or bytes to an app",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = super::webxdc_updates::instance(context, &args.positional[0]).await?;
    let data = crate::realtime::payload(&args.positional[1], args.option("--file").is_some())?;
    crate::realtime::session(context, args, id, Some(data), false).await
}
