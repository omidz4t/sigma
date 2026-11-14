use crate::{Result, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "agent-model",
    arguments: "ID_OR_ALIAS [MODEL_ID|default]",
    description: "Show or persist this native bot's model selection. Changes apply on the next provider turn across all of this bot's chats and preserve session history. IDs are validated against the provider's local model catalog. default uses the provider CLI configuration. Works online or stopped. Owner chat equivalents: /model and /model MODEL_ID/default. Custom adapters manage their own models.",
    options: "",
    example: "agent-model codex gpt-6.1-sol --account codex-new",
    summary: "Show or change a native agent's model",
};
pub(crate) async fn handle(args: &Args) -> Result<()> {
    execute(args, false).await
}
pub(crate) async fn execute(args: &Args, list: bool) -> Result<()> {
    if std::env::var_os("SIGMA_AGENT_CAPABILITY").is_some() {
        return Err(
            "model management is an owner CLI command, unavailable inside agent turns".into(),
        );
    }
    let root = crate::accounts::resolve(args, std::env::var_os("HOME").as_deref())?;
    let alias = crate::agent_registry::resolve_selector(&root, &args.positional[0])?;
    let operation = if list {
        vec!["list"]
    } else if let Some(model) = args.positional.get(1) {
        vec!["set", model]
    } else {
        vec!["get"]
    };
    let raw = if crate::agents::request(&root, &alias, "status")
        .await
        .is_ok()
    {
        crate::agents::request(&root, &alias, &format!("model {}", operation.join(" "))).await?
    } else {
        let account = crate::account::open(args).await?;
        let saved = crate::agent_registry::load(&root, &alias)?.ok_or("agent not found")?;
        let dir = crate::agent_registry::directory(&root, &alias)?;
        crate::agent_master::current(&account.context, &dir, &saved).await?;
        crate::agent_models::control(
            &dir,
            &saved.provider,
            !dir.join("custom-adapter").exists(),
            &operation,
        )?
    };
    crate::output::action_response(&raw);
    let parsed = deltachat::webxdc::StatusUpdateItem {
        payload: raw.parse()?,
        ..Default::default()
    };
    if parsed.payload["ok"].as_bool() != Some(true) {
        return Err("model operation failed".into());
    }
    Ok(())
}
