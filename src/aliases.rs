//! Account-local aliases, separate from contact display names.
use crate::Result;
use deltachat::{
    contact::{Contact, ContactId},
    context::Context,
};
use std::collections::BTreeMap;

pub(crate) fn validate(alias: &str) -> Result<()> {
    if alias.is_empty()
        || alias.len() > 64
        || !alias.as_bytes()[0].is_ascii_alphabetic()
        || !alias
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err("alias must start with a letter and contain up to 64 ASCII letters, digits, underscores or hyphens".into());
    }
    Ok(())
}

fn load(context: &Context) -> Result<BTreeMap<String, (u32, String)>> {
    let path = context.get_dbfile().with_file_name("contact-aliases.tsv");
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error.into()),
    };
    let mut aliases = BTreeMap::new();
    for line in text.lines() {
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err("invalid contact aliases file".into());
        }
        validate(parts[0])?;
        if aliases
            .insert(
                parts[0].to_owned(),
                (parts[1].parse()?, parts[2].to_owned()),
            )
            .is_some()
        {
            return Err("duplicate contact alias in saved state".into());
        }
    }
    Ok(aliases)
}

pub(crate) fn ensure_available(context: &Context, alias: &str) -> Result<()> {
    validate(alias)?;
    if load(context)?.contains_key(alias) {
        return Err(format!(
            "contact alias '{alias}' already exists; use it directly when sending"
        )
        .into());
    }
    Ok(())
}

pub(crate) async fn save(context: &Context, alias: &str, id: ContactId) -> Result<()> {
    ensure_available(context, alias)?;
    let contact = Contact::get_by_id(context, id).await?;
    let mut aliases = load(context)?;
    aliases.insert(
        alias.to_owned(),
        (id.to_u32(), contact.get_addr().to_owned()),
    );
    let text = aliases
        .iter()
        .map(|(name, (id, addr))| format!("{name}\t{id}\t{addr}\n"))
        .collect::<String>();
    let path = context.get_dbfile().with_file_name("contact-aliases.tsv");
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(&temporary)?;
    let result = (|| -> Result<()> {
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() && temporary.exists() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

pub(crate) async fn resolve(context: &Context, target: &str) -> Result<ContactId> {
    let id = if let Ok(id) = target.parse::<u32>() {
        if id == 0 {
            return Err("contact ID must be positive".into());
        }
        ContactId::new(id)
    } else {
        validate(target)?;
        let aliases = load(context)?;
        let (id, address) = aliases.get(target).ok_or_else(|| {
            format!(
                "unknown contact alias '{target}'; create it with create-contact {target} INVITE_ID"
            )
        })?;
        let id = ContactId::new(*id);
        if Contact::get_by_id(context, id).await?.get_addr() != address {
            return Err("saved alias no longer matches its contact".into());
        }
        id
    };
    Contact::get_by_id(context, id).await?;
    Ok(id)
}
