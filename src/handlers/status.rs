use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use deltachat::config::Config;
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "status",
    arguments: "",
    description: "Show transport configuration, display name, and account addresses.",
    options: "",
    example: "status",
    summary: "Show account status",
};

pub(crate) async fn handle(context: &Context, _args: &Args) -> Result<()> {
    let configured = context.is_configured().await?;
    let name = context
        .get_config(Config::Displayname)
        .await?
        .unwrap_or_default();
    let photo = context.get_config(Config::Selfavatar).await?;
    let addresses = context
        .list_transports()
        .await?
        .into_iter()
        .map(|t| t.addr)
        .collect::<Vec<_>>();
    let human = format!(
        "configured: {configured}\nname: {name}{}",
        addresses
            .iter()
            .map(|a| format!("\naddress: {a}"))
            .collect::<String>()
    );
    crate::output::record(
        "account",
        vec![
            ("configured", configured.into()),
            ("name", name.into()),
            (
                "profile_photo",
                photo.map(Into::into).unwrap_or(crate::output::Value::Null),
            ),
            (
                "addresses",
                crate::output::Value::Array(
                    addresses
                        .into_iter()
                        .map(crate::output::Value::from)
                        .collect(),
                ),
            ),
        ],
        &human,
    );
    Ok(())
}
