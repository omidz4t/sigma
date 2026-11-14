use crate::shared::managed_chat;
use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;
use deltachat::{chat, constants::Chattype};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "set-channel-description",
    arguments: "CHANNEL_ID TEXT",
    description: "Set the owned channel description. Pass a quoted empty string to clear it. Core queues updates.",
    options: "",
    example: "set-channel-description 12 \"News and updates\"",
    summary: "Set the owned channel description",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = managed_chat(context, args.positional[0].parse()?, Chattype::OutBroadcast).await?;
    let value = args.positional[1].clone();
    chat::set_chat_description(context, id, &value).await?;
    crate::out!("Channel description updated locally; run listen for pending updates");
    Ok(())
}
