use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use crate::shared::join_contact;
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "create-contact",
    arguments: "[ALIAS] SECUREJOIN_ID",
    description: "Join a full contact invitation link and print the contact ID after core completes the handshake. Optionally save an account-local alias for later sending. Aliases are case-sensitive and cannot overwrite an existing alias. Keep the inviter online. Group and channel invitations are rejected.",
    options: "  --timeout SECONDS  Wait limit in seconds (default: 120)\n",
    example: "create-contact me \"https://i.delta.chat/#FULL_INVITATION\"",
    summary: "Join a contact invitation; print its contact ID",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let alias = (args.positional.len() == 2).then(|| args.positional[0].as_str());
    if let Some(alias) = alias {
        crate::aliases::ensure_available(context, alias)?;
    }
    let invite = args.positional.last().ok_or("missing invitation")?;
    let (contact, _) = join_contact(context, invite, args.option("--timeout")).await?;
    if let Some(alias) = alias {
        crate::aliases::save(context, alias, contact).await?;
    }
    crate::output::record(
        "created",
        vec![("contact_id", contact.to_u32().into())],
        &contact.to_u32().to_string(),
    );

    Ok(())
}
