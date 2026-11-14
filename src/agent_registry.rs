//! Durable agent identities and immutable recipient bindings, scoped to an account.
use crate::{Result, account::AccountLock, cli::Args, output};
use deltachat::{
    chat::ChatId,
    contact::{Contact, ContactId},
    context::Context,
    webxdc::StatusUpdateItem,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Agent {
    pub(crate) id: u32,
    pub(crate) alias: String,
    pub(crate) provider: String,
    pub(crate) target: String,
    pub(crate) contact_id: u32,
    pub(crate) chat_id: u32,
    address: String,
    fingerprint: Option<String>,
}

pub(crate) fn directory(root: &Path, alias: &str) -> Result<PathBuf> {
    crate::aliases::validate(alias)?;
    Ok(root.join("agents").join(alias))
}
fn private_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
fn atomic_write(path: &Path, data: &str) -> Result<()> {
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
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

impl Agent {
    fn value(&self) -> output::Value {
        output::object(vec![
            ("id", self.id.into()),
            ("alias", self.alias.as_str().into()),
            ("agent", self.provider.as_str().into()),
            ("target", self.target.as_str().into()),
            ("contact_id", self.contact_id.into()),
            ("chat_id", self.chat_id.into()),
            ("address", self.address.as_str().into()),
            (
                "fingerprint",
                self.fingerprint
                    .as_deref()
                    .map(output::Value::from)
                    .unwrap_or(output::Value::Null),
            ),
        ])
    }
    pub(crate) fn stopped_status(&self, root: &Path) -> String {
        output::encode(&output::object(vec![
            ("id", self.id.into()),
            ("alias", self.alias.as_str().into()),
            ("agent", self.provider.as_str().into()),
            ("target", self.target.as_str().into()),
            ("contact_id", self.contact_id.into()),
            ("chat_id", self.chat_id.into()),
            ("data_path", root.to_string_lossy().into_owned().into()),
            ("running", false.into()),
            ("pid", output::Value::Null),
        ]))
    }
    pub(crate) async fn verify(
        &self,
        context: &Context,
        contact: ContactId,
        chat: ChatId,
    ) -> Result<()> {
        let current = Contact::get_by_id(context, contact).await?;
        if contact.to_u32() != self.contact_id
            || chat.to_u32() != self.chat_id
            || current.get_addr() != self.address
            || current.fingerprint().map(|f| f.hex()) != self.fingerprint
        {
            return Err(format!("agent '{}' (ID {}) is permanently bound to its first recipient; create a new agent alias for another person", self.alias, self.id).into());
        }
        Ok(())
    }
    pub(crate) async fn recipient(&self, context: &Context, args: &Args) -> Result<ContactId> {
        if args.option("--invite").is_some() {
            return Err("this agent is already bound; omit --invite when restarting, or use a new agent alias".into());
        }
        let contact = if let Some(target) = args.option("--to") {
            if crate::shared::is_securejoin(target) {
                if target != self.target {
                    return Err("this agent is permanently bound to its first recipient; use a new alias for another person".into());
                }
                ContactId::new(self.contact_id)
            } else {
                crate::aliases::resolve(context, target).await?
            }
        } else {
            ContactId::new(self.contact_id)
        };
        self.verify(context, contact, ChatId::new(self.chat_id))
            .await?;
        Ok(contact)
    }

    pub(crate) async fn create(
        root: &Path,
        args: &Args,
        context: &Context,
        contact: ContactId,
        chat: ChatId,
    ) -> Result<Self> {
        let alias = selected_alias(root, args)?;
        let provider = &args.positional[0];
        let parent = root.join("agents");
        private_directory(&parent)?;
        let _lock = AccountLock::acquire(parent.join("registry.lock"))?;
        if load(root, &alias)?.is_some() {
            return Err("agent alias already exists".into());
        }
        if contact.to_u32() <= 9 {
            return Err("agent recipient must be a regular contact".into());
        }
        let counter = parent.join("next-id");
        let saved_next = match std::fs::read_to_string(&counter) {
            Ok(text) => text.trim().parse::<u32>()?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 1,
            Err(error) => return Err(error.into()),
        };
        let max = list(root)?.iter().map(|a| a.id).max().unwrap_or(0);
        let id = saved_next.max(max.checked_add(1).ok_or("agent IDs exhausted")?);
        let next = id.checked_add(1).ok_or("agent IDs exhausted")?;
        let person = Contact::get_by_id(context, contact).await?;
        let agent = Self {
            id,
            alias,
            provider: provider.clone(),
            target: args
                .option("--to")
                .ok_or("a new agent requires --to TARGET")?
                .into(),
            contact_id: contact.to_u32(),
            chat_id: chat.to_u32(),
            address: person.get_addr().into(),
            fingerprint: person.fingerprint().map(|f| f.hex()),
        };
        let dir = directory(root, &agent.alias)?;
        private_directory(&dir)?;
        // Reserve the ID before saving; interrupted writes can skip an ID but never reuse it.
        atomic_write(&counter, &next.to_string())?;
        atomic_write(&dir.join("agent.json"), &output::encode(&agent.value()))?;
        Ok(agent)
    }
}

pub(crate) fn load(root: &Path, alias: &str) -> Result<Option<Agent>> {
    let path = directory(root, alias)?.join("agent.json");
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let value = StatusUpdateItem {
        payload: text.parse()?,
        ..Default::default()
    };
    let v = &value.payload;
    let string = |key| -> Result<String> {
        Ok(v.get(key)
            .and_then(|v| v.as_str())
            .ok_or("invalid agent registry string")?
            .to_owned())
    };
    let number = |key| -> Result<u32> {
        Ok(v.get(key)
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .filter(|v| *v > 0)
            .ok_or("invalid agent registry ID")?)
    };
    let fingerprint = match v.get("fingerprint") {
        Some(value) if value.is_null() => None,
        Some(value) => Some(
            value
                .as_str()
                .ok_or("invalid agent fingerprint")?
                .to_owned(),
        ),
        None => return Err("missing agent fingerprint".into()),
    };
    let agent = Agent {
        id: number("id")?,
        alias: string("alias")?,
        provider: string("agent")?,
        target: string("target")?,
        contact_id: number("contact_id")?,
        chat_id: number("chat_id")?,
        address: string("address")?,
        fingerprint,
    };
    crate::aliases::validate(&agent.provider)?;
    if agent.alias != alias || agent.contact_id <= 9 || agent.chat_id <= 9 {
        return Err("invalid agent binding".into());
    }
    Ok(Some(agent))
}

pub(crate) fn list(root: &Path) -> Result<Vec<Agent>> {
    let dir = root.join("agents");
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut agents = vec![];
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && let Some(alias) = entry.file_name().to_str()
            && crate::aliases::validate(alias).is_ok()
            && let Some(agent) = load(root, alias)?
        {
            agents.push(agent);
        }
    }
    agents.sort_by_key(|a| a.id);
    if agents.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err("duplicate saved agent IDs".into());
    }
    Ok(agents)
}

pub(crate) fn resolve_selector(root: &Path, selector: &str) -> Result<String> {
    if let Ok(id) = selector.parse::<u32>() {
        return list(root)?
            .into_iter()
            .find(|a| a.id == id)
            .map(|a| a.alias)
            .ok_or_else(|| format!("unknown agent ID: {id}").into());
    }
    crate::aliases::validate(selector)?;
    Ok(selector.into())
}

pub(crate) fn selected_alias(root: &Path, args: &Args) -> Result<String> {
    if let Some(id) = args.option("--agent-id") {
        return resolve_selector(root, id);
    }
    Ok(args
        .option("--alias")
        .unwrap_or(&args.positional[0])
        .to_owned())
}

pub(crate) fn selected(root: &Path, args: &Args) -> Result<Option<Agent>> {
    let alias = selected_alias(root, args)?;
    let agent = load(root, &alias)?;
    if let Some(agent) = &agent
        && agent.provider != args.positional[0]
    {
        return Err(format!(
            "agent '{}' uses provider {}; create a new alias to use {}",
            alias, agent.provider, args.positional[0]
        )
        .into());
    }
    Ok(agent)
}
