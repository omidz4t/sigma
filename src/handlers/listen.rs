use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::print_message;
use deltachat::context::Context;
use deltachat::{EventType, message::Message};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "listen",
    arguments: "",
    description: "Run saved chat command hooks, keep core networking running, print incoming messages/events, and retry queued sends. Ctrl-C stops I/O. Stop listen before other commands use the same account.",
    options: "  --auto-reply TEXT    Reply with this text to incoming user messages\n  --on-message PATH    Run executable with a message JSON object on stdin; stdout is reply text\n  --hook-timeout SEC   Hook time limit (default: 30, requires --on-message)\n",
    example: "listen",
    summary: "Receive and print messages until Ctrl-C",
};

pub(crate) async fn handle(context: &Context, args: &Args, root: &std::path::Path) -> Result<()> {
    if !context.is_configured().await? {
        return Err("account is not configured; retry init".into());
    }
    if let Some(path) = args.option("--on-message") {
        std::fs::canonicalize(path)?;
    }
    let mut handled = std::collections::HashSet::new();
    let mut webxdc_cursors = std::collections::HashMap::new();
    let events = context.get_event_emitter();
    context.start_io().await;
    eprintln!("Listening; press Ctrl-C to stop.");
    let result: Result<()> = async {
        loop {
            tokio::select! {
                result = tokio::signal::ctrl_c() => { result?; break; }
                event = events.recv() => match event {
                    Some(event) => {
                        if let EventType::IncomingMsg { msg_id, .. } = &event.typ {
                            let message = Message::load_from_db(context, *msg_id).await?;
                            print_message(context, &message).await?;
                            if handled.len() >= 4096 { handled.clear(); }
                            if handled.insert(message.get_id()) {
                                tokio::select! {
                                    result = async {
                                        crate::chat_hooks::incoming(context, root, &message).await?;
                                        crate::automation::incoming(context, args, &message).await
                                    } => {
                                        if let Err(error) = result { crate::output::error(&format!("automation failed for message {}: {error}", message.get_id().to_u32())); }
                                    }
                                    signal = tokio::signal::ctrl_c() => { signal?; break; }
                                }
                            }
                        } else if let EventType::WebxdcRealtimeData { msg_id, data } = &event.typ {
                            crate::realtime::print_data(*msg_id, data);
                        } else if let EventType::WebxdcStatusUpdate { msg_id, .. } = &event.typ {
                            let cursor = webxdc_cursors.entry(*msg_id).or_insert(0);
                            *cursor = super::webxdc_updates::emit(context, *msg_id, *cursor, false).await?;
                        } else {
                            crate::output::record("event", vec![("event", format!("{:?}", event.typ).into())], &format!("{:?}", event.typ));
                        }
                    },
                    None => break,
                }
            }
        }
        Ok(())
    }
    .await;
    context.stop_io().await;
    result?;

    Ok(())
}
