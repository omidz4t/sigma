use crate::{
    Result,
    cli::Args,
    help::CommandHelp,
    shared::{avatar_path, managed_chat},
};
use deltachat::{
    chat::{self, Chat},
    constants::Chattype,
    context::Context,
};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "set-group-photo",
    arguments: "GROUP_ID PATH",
    description: "Set an encrypted/group chat photo through core. Pass a quoted empty string to clear it. Direct chats and channels are rejected. Changes may queue a group update; run listen to transmit it.",
    options: "",
    example: "set-group-photo 12 ./photo.png",
    summary: "Set or clear a group photo",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = managed_chat(context, args.positional[0].parse()?, Chattype::Group).await?;
    chat::set_chat_profile_image(context, id, &avatar_path(&args.positional[1])?).await?;
    let photo = Chat::load_from_db(context, id)
        .await?
        .get_profile_image(context)
        .await?;
    crate::output::record(
        "group_photo_updated",
        vec![
            ("chat_id", id.to_u32().into()),
            (
                "profile_photo",
                photo
                    .map(|p| p.to_string_lossy().into_owned().into())
                    .unwrap_or(crate::output::Value::Null),
            ),
        ],
        "Group photo updated locally; run listen for pending updates",
    );
    Ok(())
}
