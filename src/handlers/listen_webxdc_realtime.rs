use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "listen-webxdc-realtime",
    arguments: "MESSAGE_ID",
    description: "Advertise and join this app's realtime channel, then receive live button events/data until Ctrl-C. Optional --send/--send-file sends one initial packet after joining while keeping the channel open for replies. Data is ephemeral: no replay or cursor. JSON output includes byte array data and text (null for invalid UTF-8). Stop this listener before other commands access the account.",
    options: "  --send TEXT        Send initial UTF-8 text after joining\n  --send-file PATH   Send initial binary bytes instead of text\n  --timeout SECONDS  Connection wait limit (default: 120)\n",
    example: "listen-webxdc-realtime 83 --send 'Hello live!' --json",
    summary: "Join an app's realtime channel and listen",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = super::webxdc_updates::instance(context, &args.positional[0]).await?;
    let initial = match (args.option("--send"), args.option("--send-file")) {
        (Some(text), None) => Some(crate::realtime::payload(text, false)?),
        (None, Some(path)) => Some(crate::realtime::payload(path, true)?),
        (None, None) => None,
        _ => return Err("--send and --send-file are mutually exclusive".into()),
    };
    crate::realtime::session(context, args, id, initial, true).await
}
