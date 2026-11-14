use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "agent-master",
    arguments: "ID_OR_ALIAS [CONTACT_ID]",
    description: "Show or transfer the sole bot master. First --to initializes mastery; only the current master can transfer it in chat using /master (reply to a person's message) or /master CONTACT_ID privately. The trusted terminal can transfer or repair mastery online or stopped. Transfer clears delegated approvals and invalidates old-master actions; it does not share private session history.",
    options: "",
    example: "agent-master codex 15 --account codex-new",
    summary: "Show or change an agent's sole bot master",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    if std::env::var_os("SIGMA_AGENT_CAPABILITY").is_some() {
        return Err("bot master management is unavailable inside agent turns".into());
    }
    let root = crate::accounts::resolve(args, std::env::var_os("HOME").as_deref())?;
    let alias = crate::agent_registry::resolve_selector(&root, &args.positional[0])?;
    let operation = if let Some(id) = args.positional.get(1) {
        vec!["set", id]
    } else {
        vec!["get"]
    };
    let raw = if crate::agents::request(&root, &alias, "status")
        .await
        .is_ok()
    {
        crate::agents::request(&root, &alias, &format!("master {}", operation.join(" "))).await?
    } else {
        let account = crate::account::open(args).await?;
        let saved = crate::agent_registry::load(&root, &alias)?.ok_or("agent not found")?;
        crate::agent_master::control(
            &account.context,
            &crate::agent_registry::directory(&root, &alias)?,
            &saved,
            &operation,
        )
        .await?
    };
    crate::output::action_response(&raw);
    let parsed = deltachat::webxdc::StatusUpdateItem {
        payload: raw.parse()?,
        ..Default::default()
    };
    if parsed.payload["ok"].as_bool() != Some(true) {
        return Err("bot master operation failed".into());
    }
    Ok(())
}
