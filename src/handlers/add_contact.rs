use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use deltachat::contact::Contact;
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "add-contact",
    arguments: "ADDRESS",
    description: "Create an address contact and print its numeric contact ID.",
    options: "  --name NAME  Set the contact display name\n",
    example: "add-contact alice@example.org --name Alice",
    summary: "Add a contact; print its ID",
};

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = Contact::create(
        context,
        args.option("--name").unwrap_or(""),
        &args.positional[0],
    )
    .await?;
    crate::output::record(
        "created",
        vec![("contact_id", id.to_u32().into())],
        &id.to_u32().to_string(),
    );

    Ok(())
}
