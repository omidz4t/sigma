use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use deltachat::contact::Contact;
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "contacts",
    arguments: "",
    description: "List locally stored key and address contacts with their numeric IDs.",
    options: "",
    example: "contacts",
    summary: "List contacts (ID, name, address)",
};

pub(crate) async fn handle(context: &Context, _args: &Args) -> Result<()> {
    for flags in [0, deltachat::constants::DC_GCL_ADDRESS] {
        for id in Contact::get_all(context, flags, None).await? {
            let contact = Contact::get_by_id(context, id).await?;
            crate::output::record(
                "contact",
                vec![
                    ("contact_id", id.to_u32().into()),
                    ("name", contact.get_name().into()),
                    ("address", contact.get_addr().into()),
                    ("key_contact", contact.is_key_contact().into()),
                ],
                &format!(
                    "{}\t{:?}\t{}",
                    id.to_u32(),
                    contact.get_name(),
                    contact.get_addr()
                ),
            );
        }
    }

    Ok(())
}
