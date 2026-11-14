use crate::{Result, account::Account, cli::Args, help::CommandHelp};
pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "app-bind",
    arguments: "NAME MESSAGE_ID",
    description: "Save a name for an existing webxdc app message. Does not resend the app. Names are unique within this account; an existing name is never overwritten.",
    options: "",
    example: "app-bind demo 95",
    summary: "Save a name for an existing webxdc app message",
};
pub(crate) async fn handle(account: &Account, args: &Args) -> Result<()> {
    crate::apps::bind(
        &account.context,
        &account.path,
        &args.positional[0],
        deltachat::message::MsgId::new(args.positional[1].parse()?),
    )
    .await?;
    Ok(())
}
