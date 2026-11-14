use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::{config::Config, context::Context};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "set-profile-photo",
    arguments: "PATH",
    description: "Set this bot's profile photo from a local image through core. Pass a quoted empty string to clear it. Core resizes and stores the image; subsequent outgoing messages propagate it to contacts. Does not provision or change account identity.",
    options: "",
    example: "set-profile-photo ./photo.png",
    summary: "Set or clear this bot's profile photo",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let path = crate::shared::avatar_path(&args.positional[0])?;
    context
        .set_config(
            Config::Selfavatar,
            if path.is_empty() { None } else { Some(&path) },
        )
        .await?;
    let photo = context.get_config(Config::Selfavatar).await?;
    crate::output::record(
        "profile_photo_updated",
        vec![(
            "profile_photo",
            photo.map(Into::into).unwrap_or(crate::output::Value::Null),
        )],
        "Bot profile photo updated; send a message to share it with contacts",
    );
    Ok(())
}
