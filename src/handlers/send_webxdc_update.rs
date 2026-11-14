use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "send-webxdc-update",
    arguments: "MESSAGE_ID UPDATE_JSON",
    description: "Queue shared state for an existing webxdc app using core. UPDATE_JSON must be an object containing payload, with optional info, summary, document, href or notify fields. Use info for a visible chat entry and notify for a user notification; omit both for a silent update. Notification display follows client settings. This updates app state, not the .xdc code. Run listen or listen-webxdc-updates afterward to transmit queued updates. MESSAGE_ID is the original app message's local ID.",
    options: "",
    example: "send-webxdc-update 26 '{\"payload\":{\"text\":\"Hello from CLI\"}}'",
    summary: "Queue a webxdc app state update",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = super::webxdc_updates::instance(context, &args.positional[0]).await?;
    // Core parses and validates the update and chat permissions before queuing it.
    context
        .send_webxdc_status_update(id, &args.positional[1])
        .await?;
    crate::output::record(
        "webxdc_update_queued",
        vec![("message_id", id.to_u32().into())],
        &format!(
            "App update queued for message {}; run listen to transmit it",
            id.to_u32()
        ),
    );
    Ok(())
}
