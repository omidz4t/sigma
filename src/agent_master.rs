//! Exactly one bot master. Original --to remains provenance, not permanent authority.
use crate::{
    Result,
    agent_registry::Agent,
    output::{self, Value},
};
use deltachat::{
    chat::{self, ChatId},
    contact::{Contact, ContactId},
    context::Context,
    message::Message,
    webxdc::StatusUpdateItem,
};
use std::path::Path;
#[derive(Clone, Debug)]
pub(crate) struct Master {
    pub(crate) contact: ContactId,
    pub(crate) chat: ChatId,
    address: String,
    fingerprint: Option<String>,
}
impl Master {
    async fn from_contact(context: &Context, contact: ContactId, chat: ChatId) -> Result<Self> {
        let person = Contact::get_by_id(context, contact).await?;
        Ok(Self {
            contact,
            chat,
            address: person.get_addr().into(),
            fingerprint: person.fingerprint().map(|f| f.hex()),
        })
    }
    pub(crate) async fn verify(&self, context: &Context) -> Result<()> {
        let current = Self::from_contact(context, self.contact, self.chat).await?;
        if current.address != self.address
            || current.fingerprint != self.fingerprint
            || !chat::is_contact_in_chat(context, self.chat, self.contact).await?
        {
            return Err("bot master identity changed; repair it from the owner terminal".into());
        }
        Ok(())
    }
    fn value(&self) -> Value {
        output::object(vec![
            ("contact_id", self.contact.to_u32().into()),
            ("chat_id", self.chat.to_u32().into()),
            ("address", self.address.clone().into()),
            (
                "fingerprint",
                self.fingerprint
                    .as_deref()
                    .map(Value::from)
                    .unwrap_or(Value::Null),
            ),
        ])
    }
}
pub(crate) fn stored(dir: &Path) -> Result<Option<Master>> {
    let raw = match std::fs::read_to_string(dir.join("master.json")) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let item = StatusUpdateItem {
        payload: raw.parse()?,
        ..Default::default()
    };
    let v = &item.payload;
    let id = |key: &str| -> Result<u32> {
        Ok(v[key]
            .as_u64()
            .and_then(|id| u32::try_from(id).ok())
            .filter(|id| *id > 9)
            .ok_or("invalid bot master ID")?)
    };
    let fingerprint = match v.get("fingerprint") {
        Some(v) if v.is_null() => None,
        Some(v) => Some(v.as_str().ok_or("invalid master fingerprint")?.into()),
        None => return Err("missing master fingerprint".into()),
    };
    Ok(Some(Master {
        contact: ContactId::new(id("contact_id")?),
        chat: ChatId::new(id("chat_id")?),
        address: v["address"]
            .as_str()
            .ok_or("missing master address")?
            .into(),
        fingerprint,
    }))
}
pub(crate) async fn current(context: &Context, dir: &Path, agent: &Agent) -> Result<Master> {
    let master = match stored(dir)? {
        Some(master) => master,
        None => {
            agent
                .verify(
                    context,
                    ContactId::new(agent.contact_id),
                    ChatId::new(agent.chat_id),
                )
                .await?;
            Master::from_contact(
                context,
                ContactId::new(agent.contact_id),
                ChatId::new(agent.chat_id),
            )
            .await?
        }
    };
    master.verify(context).await?;
    Ok(master)
}
pub(crate) async fn matches(
    context: &Context,
    dir: &Path,
    contact: ContactId,
    chat: ChatId,
) -> Result<bool> {
    if let Some(master) = stored(dir)? {
        if master.contact != contact || master.chat != chat {
            return Ok(false);
        }
        master.verify(context).await?;
    }
    Ok(true)
}
fn write(dir: &Path, name: &str, raw: &str) -> Result<()> {
    let temp = dir.join(format!("{name}.{}.tmp", uuid::Uuid::new_v4()));
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    use std::io::Write;
    let mut file = opts.open(&temp)?;
    file.write_all(raw.as_bytes())?;
    file.sync_all()?;
    std::fs::rename(temp, dir.join(name))?;
    Ok(())
}
pub(crate) async fn transfer(
    context: &Context,
    dir: &Path,
    agent: &Agent,
    actor: Option<ContactId>,
    target: ContactId,
) -> Result<Master> {
    let _lock = crate::agent_guard::POLICY_LOCK.lock().await;
    // Terminal administration can repair changed keys; chat administration must verify the old master.
    if let Some(actor) = actor
        && current(context, dir, agent).await?.contact != actor
    {
        return Err("only the current bot master may transfer mastery".into());
    }
    if target.to_u32() <= 9 || Contact::is_blocked_load(context, target).await? {
        return Err("choose a regular, unblocked key contact".into());
    }
    if let Some(prior) = stored(dir)?
        && prior.contact == target
        && prior.verify(context).await.is_ok()
    {
        return Err("that person is already the bot master".into());
    }
    let chat = ChatId::create_for_contact(context, target).await?;
    let master = Master::from_contact(context, target, chat).await?;
    if master.fingerprint.is_none() {
        return Err("the new bot master must be a key contact with a fingerprint".into());
    }
    // Clear delegations first: interruption can reduce privileges but cannot grant old approvals to the new master.
    write(dir, "approvals.json", "{}")?;
    write(dir, "master.json", &output::encode(&master.value()))?;
    Ok(master)
}
pub(crate) async fn control(
    context: &Context,
    dir: &Path,
    agent: &Agent,
    args: &[&str],
) -> Result<String> {
    let master = match args {
        ["get"] => current(context, dir, agent).await?,
        ["set", id] => transfer(context, dir, agent, None, ContactId::new(id.parse()?)).await?,
        _ => return Err("expected get or set CONTACT_ID".into()),
    };
    Ok(output::encode(&output::object(vec![
        ("ok", true.into()),
        ("bot_master", master.value()),
    ])))
}
pub(crate) async fn chat(
    context: &Context,
    scope: &crate::agent_guard::Scope,
    text: &str,
    parent: Option<(ChatId, ContactId, bool)>,
) -> Result<String> {
    let root = scope
        .dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("agent root unavailable")?;
    let alias = scope
        .dir
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("agent alias unavailable")?;
    let agent = crate::agent_registry::load(root, alias)?.ok_or("agent missing")?;
    let parts = text.split_whitespace().collect::<Vec<_>>();
    let target=match parts.as_slice(){["/master"]=>match parent{Some((chat,person,false)) if chat==scope.chat=>Some(person),_=>None},["/master",id] if scope.chat==scope.owner_chat=>Some(ContactId::new(id.parse().map_err(|_|"use /master CONTACT_ID in the bot master's private chat")?)),_=>return Ok("Use /master to show the bot master, reply /master to a person's message in a group, or /master CONTACT_ID privately to transfer mastery.".into())};
    let Some(target) = target else {
        return Ok(format!(
            "Bot master: contact {}. Only one person is bot master. Reply /master to the new person's message, or /master CONTACT_ID privately, to transfer. This clears all delegated approvals.",
            scope.owner.to_u32()
        ));
    };
    let master = transfer(context, &scope.dir, &agent, Some(scope.sender), target).await?;
    let mut notice=Message::new_text("You are now this bot's sole bot master. Existing delegated approvals were cleared. Use /access, /approve, /revoke, /models and /model to manage it.".into());
    if let Err(e) = chat::send_msg(context, master.chat, &mut notice).await {
        crate::output::error(&format!("Master transferred but notification failed: {e}"));
    }
    Ok(format!(
        "Bot master transferred to contact {}. Previous delegated approvals were cleared; the former master has no automatic access. Private conversation history is kept separate.",
        master.contact.to_u32()
    ))
}
