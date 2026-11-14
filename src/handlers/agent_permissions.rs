use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "agent-permissions",
    arguments: "ID_OR_ALIAS list | ID_OR_ALIAS approve|assign|revoke CHAT_ID CONTACT_ID",
    description: "Manage this agent's sender allowlist from the local owner CLI, online or stopped. Approvals are scoped to a chat and contact fingerprint. In groups the current bot master must remain a member. Only an explicit /master or agent-master transfer changes the sole bot master. Chat /approve, /assign and /revoke are owner-only replies to another person's message. Owner-only /access lists permissions in chat and /access remove CONTACT_ID removes access (all this bot's chats privately, current group otherwise). The model has no permission-management action.",
    options: "",
    example: "agent-permissions codex approve 12 15 --account codex-new",
    summary: "List or change an agent's sender permissions",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    if std::env::var_os("SIGMA_AGENT_CAPABILITY").is_some() {
        return Err(
            "permission management is an owner CLI command, unavailable inside agent turns".into(),
        );
    }
    let root = crate::accounts::resolve(args, std::env::var_os("HOME").as_deref())?;
    let alias = crate::agent_registry::resolve_selector(&root, &args.positional[0])?;
    let parts = args.positional[1..]
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let raw = if crate::agents::request(&root, &alias, "status")
        .await
        .is_ok()
    {
        crate::agents::request(&root, &alias, &format!("permissions {}", parts.join(" "))).await?
    } else {
        let account = crate::account::open(args).await?;
        let saved = crate::agent_registry::load(&root, &alias)?.ok_or("agent not found")?;
        let master = crate::agent_master::current(
            &account.context,
            &crate::agent_registry::directory(&root, &alias)?,
            &saved,
        )
        .await?;
        let owner = master.contact;
        let chat = master.chat;
        crate::agent_guard::control(
            &account.context,
            &crate::agent_registry::directory(&root, &alias)?,
            owner,
            chat,
            &parts,
        )
        .await?
    };
    crate::output::action_response(&raw);
    let parsed = deltachat::webxdc::StatusUpdateItem {
        payload: raw.parse()?,
        ..Default::default()
    };
    if parsed.payload["ok"].as_bool() != Some(true) {
        return Err("permission operation failed".into());
    }
    Ok(())
}
