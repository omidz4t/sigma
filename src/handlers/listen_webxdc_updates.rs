use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::{EventType, context::Context};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "listen-webxdc-updates",
    arguments: "MESSAGE_ID",
    description: "Replay stored updates after the cursor, then receive and print new updates for this app, including button events emitted with webxdc.sendUpdate(). Keeps core networking running and transmits queued updates. Ctrl-C stops. Each result contains updates and last_serial; save that local cursor to resume. Stop this listener before using another command with the same account.",
    options: "  --after SERIAL     Resume after this local update serial (default: 0)\n",
    example: "listen-webxdc-updates 26 --after 0 --json",
    summary: "Listen for app state changes and button events",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    crate::shared::ensure_configured(context).await?;
    let id = super::webxdc_updates::instance(context, &args.positional[0]).await?;
    let events = context.get_event_emitter();
    let mut cursor = super::webxdc_updates::emit(
        context,
        id,
        args.option("--after").unwrap_or("0").parse()?,
        true,
    )
    .await?;
    context.start_io().await;
    eprintln!("Listening for webxdc updates; press Ctrl-C to stop.");
    let result: Result<()> = async {
        loop {
            tokio::select! {
                signal = tokio::signal::ctrl_c() => { signal?; break; }
                event = events.recv() => match event {
                    Some(event) => if let EventType::WebxdcStatusUpdate { msg_id, .. } = event.typ && msg_id == id {
                            cursor = super::webxdc_updates::emit(context, id, cursor, false).await?;
                    },
                    None => break,
                }
            }
        }
        Ok(())
    }
    .await;
    context.stop_io().await;
    result
}
