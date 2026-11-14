use crate::shared::managed_chat;
use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::context::Context;
use deltachat::{chat, constants::Chattype};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "set-channel-photo",
    arguments: "CHANNEL_ID PATH",
    description: "Set the owned channel photo. Pass a quoted empty string to clear it. Core queues updates.",
    options: "",
    example: "set-channel-photo 12 ./photo.jpg",
    summary: "Set the owned channel photo",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = managed_chat(context, args.positional[0].parse()?, Chattype::OutBroadcast).await?;
    let value = if args.positional[1].is_empty() {
        String::new()
    } else {
        std::fs::canonicalize(&args.positional[1])?
            .to_str()
            .ok_or("photo path must be UTF-8")?
            .to_owned()
    };
    chat::set_chat_profile_image(context, id, &value).await?;
    crate::out!("Channel photo updated locally; run listen for pending updates");
    Ok(())
}
