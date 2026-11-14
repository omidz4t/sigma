use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "agent-calls",
    arguments: "ID_OR_ALIAS [COUNT]",
    description: "List account call history plus this agent's queued and failed outbound requests, newest first. Shows call/request IDs, status, connected duration in seconds and failure reason. Includes archived chats. Works online through the worker or offline. COUNT defaults to 50, maximum 100.",
    options: "",
    example: "agent-calls codex --account codex-new --json",
    summary: "List calls, duration and failed/queued requests",
};
pub(crate) const CANCEL_HELP: CommandHelp = CommandHelp {
    name: "agent-call-cancel",
    arguments: "ID_OR_ALIAS CALL_ID_OR_REQUEST_UUID_OR_all",
    description: "Cancel a queued request by UUID or end a ringing/active call by numeric message ID. Use all to cancel this agent's requests and all active calls in the selected account. Requires the trusted terminal; this is not an agent tool. Repeated cancellation is harmless.",
    options: "",
    example: "agent-call-cancel codex all --account codex-new",
    summary: "Cancel queued, ringing or active calls",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    if std::env::var_os("SIGMA_AGENT_CAPABILITY").is_some() {
        return Err("call management requires the trusted terminal".into());
    }
    let root = crate::accounts::resolve(args, std::env::var_os("HOME").as_deref())?;
    let alias = crate::agent_registry::resolve_selector(&root, &args.positional[0])?;
    let parts = if args.command == "agent-calls" {
        vec![
            "list",
            args.positional.get(1).map(String::as_str).unwrap_or("50"),
        ]
    } else {
        vec!["cancel", args.positional[1].as_str()]
    };
    let raw = if crate::agents::request(&root, &alias, "status")
        .await
        .is_ok()
    {
        crate::agents::request(&root, &alias, &format!("calls {}", parts.join(" "))).await?
    } else {
        let account = crate::account::open(args).await?;
        let raw = crate::call_history::control(
            &account.context,
            &crate::agent_registry::directory(&root, &alias)?,
            &parts,
        )
        .await?;
        if parts[0] == "cancel" {
            account.context.start_io().await;
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                account.context.wait_for_all_work_done(),
            )
            .await;
            account.context.stop_io().await;
        }
        raw
    };
    let parsed = deltachat::webxdc::StatusUpdateItem {
        payload: raw.parse()?,
        ..Default::default()
    };
    if crate::output::is_json() || parsed.payload["calls"].as_array().is_none() {
        crate::output::action_response(&raw);
    } else {
        for call in parsed.payload["calls"].as_array().unwrap() {
            crate::out!(
                "call={} request={} status={} stage={} duration={}s chat={} time={} recording={} {}",
                call["call_id"],
                call["request_id"].as_str().unwrap_or("-"),
                call["status"].as_str().unwrap_or("unknown"),
                call["last_stage"].as_str().unwrap_or("unknown"),
                call["duration_seconds"].as_i64().unwrap_or(0),
                call["chat_id"],
                deltachat::tools::timestamp_to_str(
                    call["created_at"]
                        .as_i64()
                        .or(call["timestamp"].as_i64())
                        .unwrap_or(0)
                ),
                call["recording"]["path"].as_str().unwrap_or("-"),
                call["error"].as_str().unwrap_or("")
            );
        }
    }
    if parsed.payload["ok"].as_bool() != Some(true) {
        return Err("call management failed".into());
    }
    Ok(())
}
