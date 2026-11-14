use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use deltachat::context::Context;
use deltachat::{chat::Chat, chatlist::Chatlist};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "chats",
    arguments: "",
    description: "List locally stored chats and channels with their numeric chat IDs.",
    options: "",
    example: "chats",
    summary: "List chats (ID, name)",
};

pub(crate) async fn handle(context: &Context, _args: &Args) -> Result<()> {
    let list = Chatlist::try_load(context, 0, None, None).await?;
    for index in 0..list.len() {
        let id = list.get_chat_id(index)?;
        let chat = Chat::load_from_db(context, id).await?;
        let photo = chat.get_profile_image(context).await?;
        crate::output::record(
            "chat",
            vec![
                ("chat_id", id.to_u32().into()),
                ("name", chat.get_name().into()),
                ("chat_type", format!("{:?}", chat.get_type()).into()),
                (
                    "profile_photo",
                    photo
                        .map(|p| p.to_string_lossy().into_owned().into())
                        .unwrap_or(crate::output::Value::Null),
                ),
            ],
            &format!("{}\t{:?}", id.to_u32(), chat.get_name()),
        );
    }

    Ok(())
}
