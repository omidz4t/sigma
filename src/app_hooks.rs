//! Explicit app hooks: saved configuration, durable pending events, bounded execution.
use crate::{Result, apps::App, cli::Args, output::Value};
use deltachat::{context::Context, message::MsgId, webxdc::StatusUpdateItem};
use std::{
    collections::{HashSet, VecDeque},
    path::{Path, PathBuf},
};

pub(crate) const QUEUE_LIMIT: usize = 256;
#[derive(Clone, Debug)]
pub(crate) struct Hook {
    pub id: String,
    pub path: PathBuf,
    pub event: String,
    pub match_json: Option<String>,
    pub timeout: u64,
    pub after_seq: i64,
}
impl Hook {
    pub(crate) fn value(&self) -> Value {
        crate::output::object(vec![
            ("id", self.id.clone().into()),
            ("path", self.path.to_string_lossy().into_owned().into()),
            ("event", self.event.clone().into()),
            ("type", kind_name(&self.event).into()),
            (
                "match_json",
                self.match_json
                    .clone()
                    .map(Into::into)
                    .unwrap_or(Value::Null),
            ),
            ("timeout", (self.timeout as i64).into()),
            ("after_seq", self.after_seq.into()),
        ])
    }
    pub(crate) fn parse(raw: &str) -> Result<Self> {
        let v = StatusUpdateItem {
            payload: raw.parse()?,
            ..Default::default()
        };
        let text = |key| -> Result<String> {
            Ok(v.payload
                .get(key)
                .and_then(|x| x.as_str())
                .ok_or("invalid saved hook field")?
                .into())
        };
        Ok(Self {
            id: text("id")?,
            path: PathBuf::from(text("path")?),
            event: text("event")?,
            match_json: v
                .payload
                .get("match_json")
                .and_then(|x| x.as_str())
                .map(str::to_owned),
            timeout: v
                .payload
                .get("timeout")
                .and_then(|x| x.as_u64())
                .filter(|n| *n > 0)
                .ok_or("invalid hook timeout")?,
            after_seq: v
                .payload
                .get("after_seq")
                .and_then(|x| x.as_i64())
                .ok_or("invalid hook boundary")?,
        })
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Job {
    pub seq: i64,
    pub input: String,
    pub hook: Hook,
}
pub(crate) type Pending = VecDeque<Job>;
pub(crate) fn requested(args: &Args) -> bool {
    args.option("--on-event").is_some() || args.option("--no-hook").is_some()
}
pub(crate) fn executable(raw: &str) -> Result<PathBuf> {
    let path = std::fs::canonicalize(raw)?;
    if !path.is_file() {
        return Err("app hook must be an executable regular file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if std::fs::metadata(&path)?.permissions().mode() & 0o111 == 0 {
            return Err("app hook is not executable; chmod +x it first".into());
        }
    }
    Ok(path)
}
pub(crate) fn normalize(event: &str) -> Result<&'static str> {
    match event {
        "realtime" | "realtime_received" => Ok("realtime_received"),
        "update" | "webxdc_updates" => Ok("webxdc_updates"),
        "message" | "message_received" => Ok("message_received"),
        "command" | "command_received" => Ok("command_received"),
        "all" => Ok("all"),
        _ => Err("hook type must be realtime, update, message or command (legacy --event all is also supported)".into()),
    }
}
pub(crate) fn kind_name(event: &str) -> &'static str {
    match normalize(event) {
        Ok("realtime_received") => "realtime",
        Ok("webxdc_updates") => "update",
        Ok("message_received") => "message",
        Ok("command_received") => "command",
        _ => "all",
    }
}
pub(crate) fn needs_realtime(hooks: &[Hook]) -> bool {
    hooks
        .iter()
        .any(|h| matches!(normalize(&h.event), Ok("all" | "realtime_received")))
}
pub(crate) fn command(text: &str) -> Option<(String, String)> {
    let text = text.trim();
    let end = text.find(char::is_whitespace).unwrap_or(text.len());
    let name = &text[..end];
    if name.len() <= 1
        || !name.starts_with('/')
        || !name[1..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_@-".contains(c))
    {
        return None;
    }
    Some((name.into(), text[end..].trim_start().into()))
}
pub(crate) fn selectors(args: &Args) -> Result<(String, Option<String>)> {
    if args.option("--type").is_some() && args.option("--event").is_some() {
        return Err("choose --type or legacy --event, not both".into());
    }
    if let Some(kind) = args.option("--type")
        && !matches!(kind, "realtime" | "update" | "message" | "command")
    {
        return Err("--type must be realtime, update, message or command".into());
    }
    let event = normalize(
        args.option("--type")
            .or_else(|| args.option("--event"))
            .ok_or("app-hook-add requires --type realtime|update|message|command (or --event)")?,
    )?;
    let mut filter = args.option("--match").map(str::to_owned);
    validate_selector(event, filter.as_deref())?;
    if let Some(name) = args.option("--command") {
        if event != "command_received"
            || command(name).is_none_or(|(cmd, rest)| cmd != name || !rest.is_empty())
        {
            return Err(
                "--command requires a command hook and a single slash command such as /ping".into(),
            );
        }
        let mut value = StatusUpdateItem {
            payload: filter.as_deref().unwrap_or("{}").parse()?,
            ..Default::default()
        };
        if value
            .payload
            .get("command")
            .is_some_and(|v| v.as_str() != Some(name))
        {
            return Err("--command conflicts with the command in --match".into());
        }
        value.payload["command"] = name.into();
        filter = Some(value.payload.to_string());
    }
    Ok((event.into(), filter))
}
pub(crate) fn validate_selector(event: &str, filter: Option<&str>) -> Result<()> {
    normalize(event)?;
    if let Some(raw) = filter {
        let v = StatusUpdateItem {
            payload: raw.parse()?,
            ..Default::default()
        };
        if !v.payload.is_object() {
            return Err("--match must be a JSON object describing a payload subset".into());
        }
    }
    Ok(())
}
fn legacy(app: &mut App) {
    let default = app.hooks.iter().find(|h| h.id == "default");
    app.hook_path = default.map(|h| h.path.clone());
    app.hook_timeout = default.map_or(30, |h| h.timeout);
    app.hook_after_seq = default.map_or(0, |h| h.after_seq);
}
pub(crate) fn add_with_journal(
    app: &mut App,
    root: &Path,
    journal: &mut crate::apps::Journal,
    mut hook: Hook,
) -> Result<()> {
    crate::apps::validate_name(&hook.id)?;
    if app.hooks.iter().any(|h| h.id == hook.id) {
        return Err(format!(
            "hook {} already exists; remove it before adding a replacement",
            hook.id
        )
        .into());
    }
    validate_selector(&hook.event, hook.match_json.as_deref())?;
    hook.event = normalize(&hook.event)?.into();
    if hook.timeout == 0 || hook.timeout > i64::MAX as u64 {
        return Err("hook timeout must be positive and fit a signed 64-bit integer".into());
    }
    hook.path = executable(hook.path.to_str().ok_or("hook path must be UTF-8")?)?;
    hook.after_seq = journal.append(
        "hook_added",
        vec![
            ("hook_id", hook.id.clone().into()),
            ("event", hook.event.clone().into()),
        ],
        None,
    )?;
    app.hooks.push(hook);
    legacy(app);
    app.save(root)
}
pub(crate) fn add(
    app: &mut App,
    root: &Path,
    id: &str,
    path: &str,
    event: &str,
    filter: Option<&str>,
    timeout: u64,
) -> Result<()> {
    let mut journal = crate::apps::Journal::open(root, app)?;
    add_with_journal(
        app,
        root,
        &mut journal,
        Hook {
            id: id.into(),
            path: PathBuf::from(path),
            event: event.into(),
            match_json: filter.map(str::to_owned),
            timeout,
            after_seq: 0,
        },
    )
}
pub(crate) fn remove_with_journal(
    app: &mut App,
    root: &Path,
    journal: &mut crate::apps::Journal,
    id: &str,
) -> Result<()> {
    if !app.hooks.iter().any(|h| h.id == id) {
        return Err(format!("hook {id} does not exist").into());
    }
    journal.append("hook_removed", vec![("hook_id", id.into())], None)?;
    app.hooks.retain(|h| h.id != id);
    legacy(app);
    app.save(root)
}
pub(crate) fn remove(app: &mut App, root: &Path, id: &str) -> Result<()> {
    let mut journal = crate::apps::Journal::open(root, app)?;
    remove_with_journal(app, root, &mut journal, id)
}
pub(crate) fn configure(app: &mut App, root: &Path, args: &Args) -> Result<()> {
    if !requested(args) {
        return Ok(());
    }
    if let Some(path) = args.option("--on-event") {
        // Validate before replacing the compatibility/default hook.
        executable(path)?;
        if app.hooks.iter().any(|h| h.id == "default") {
            remove(app, root, "default")?;
        }
        add(
            app,
            root,
            "default",
            path,
            "all",
            None,
            args.option("--hook-timeout").unwrap_or("30").parse()?,
        )
    } else {
        let ids = app.hooks.iter().map(|h| h.id.clone()).collect::<Vec<_>>();
        for id in ids {
            remove(app, root, &id)?;
        }
        Ok(())
    }
}
fn subset(expected: &str, actual: &str) -> Result<bool> {
    let a = StatusUpdateItem {
        payload: expected.parse()?,
        ..Default::default()
    };
    let b = StatusUpdateItem {
        payload: actual.parse()?,
        ..Default::default()
    };
    if let Some(object) = a.payload.as_object() {
        for (key, value) in object {
            let Some(other) = b.payload.get(key) else {
                return Ok(false);
            };
            if !subset(&value.to_string(), &other.to_string())? {
                return Ok(false);
            }
        }
        return Ok(b.payload.is_object());
    }
    Ok(a.payload == b.payload)
}
pub(crate) fn select(hook: &Hook, seq: i64, line: &str) -> Result<Option<Job>> {
    if seq <= hook.after_seq {
        return Ok(None);
    }
    let mut v = StatusUpdateItem {
        payload: line.parse()?,
        ..Default::default()
    };
    let kind = v.payload.get("type").and_then(|x| x.as_str()).unwrap_or("");
    if !matches!(
        kind,
        "realtime_received" | "webxdc_updates" | "message_received" | "command_received"
    ) || (hook.event == "all" && !matches!(kind, "realtime_received" | "webxdc_updates"))
        || (hook.event != "all" && normalize(&hook.event)? != kind)
    {
        return Ok(None);
    }
    if let Some(filter) = &hook.match_json {
        if kind == "realtime_received" {
            let Some(text) = v.payload.get("text").and_then(|x| x.as_str()) else {
                return Ok(None);
            };
            if !subset(filter, text).unwrap_or(false) {
                return Ok(None);
            }
        } else if kind == "webxdc_updates" {
            let mut matched = Vec::new();
            if let Some(updates) = v.payload.get("updates").and_then(|x| x.as_array()) {
                for update in updates {
                    if let Some(payload) = update.get("payload")
                        && subset(filter, &payload.to_string())?
                    {
                        matched.push(update.clone());
                    }
                }
            }
            if matched.is_empty() {
                return Ok(None);
            }
            v.payload["updates"] = matched.into();
        } else if !subset(filter, &v.payload.to_string())? {
            return Ok(None);
        }
    }
    v.payload["hook_id"] = hook.id.clone().into();
    Ok(Some(Job {
        seq,
        input: v.payload.to_string(),
        hook: hook.clone(),
    }))
}
pub(crate) fn recover(path: &Path, hooks: &[Hook]) -> Result<Pending> {
    let raw = std::fs::read_to_string(path)?;
    let mut events = Vec::new();
    let mut finished = HashSet::new();
    for line in raw.lines() {
        let v = StatusUpdateItem {
            payload: line.parse()?,
            ..Default::default()
        };
        let seq = v
            .payload
            .get("seq")
            .and_then(|x| x.as_i64())
            .ok_or("missing event sequence")?;
        if v.payload.get("type").and_then(|x| x.as_str()) == Some("hook_finished") {
            if let Some(event_seq) = v.payload.get("event_seq").and_then(|x| x.as_i64()) {
                finished.insert((
                    event_seq,
                    v.payload
                        .get("hook_id")
                        .and_then(|x| x.as_str())
                        .unwrap_or("default")
                        .to_owned(),
                ));
            }
        } else {
            events.push((seq, line));
        }
    }
    let mut pending = VecDeque::new();
    for (seq, line) in events {
        for hook in hooks {
            if !finished.contains(&(seq, hook.id.clone()))
                && let Some(job) = select(hook, seq, line)?
            {
                pending.push_back(job);
            }
        }
    }
    Ok(pending)
}

pub(crate) fn spawn(
    path: PathBuf,
    input: String,
    state_path: PathBuf,
    timeout: u64,
) -> tokio::task::JoinHandle<Result<String>> {
    tokio::spawn(async move {
        let mut value = StatusUpdateItem {
            payload: input.parse()?,
            ..Default::default()
        };
        value.payload["state_path"] = state_path.to_string_lossy().into_owned().into();
        crate::automation::run_hook(&path, &value.payload.to_string(), timeout).await
    })
}

/// Hook output is an explicit JSON action object, never a shell command.
#[cfg(test)]
pub(crate) async fn actions(context: &Context, id: MsgId, raw: &str, joined: bool) -> Result<()> {
    actions_for_event(context, id, raw, joined, "{}").await
}
pub(crate) async fn actions_for_event(
    context: &Context,
    id: MsgId,
    raw: &str,
    joined: bool,
    input: &str,
) -> Result<()> {
    if raw.trim().is_empty() {
        return Ok(());
    }
    let value = StatusUpdateItem {
        payload: raw.parse()?,
        ..Default::default()
    };
    let object = value
        .payload
        .as_object()
        .ok_or("hook stdout must be a JSON object with optional send/update/reply fields")?;
    if object
        .keys()
        .any(|key| key != "send" && key != "update" && key != "reply")
    {
        return Err("unknown hook action; supported fields: send, update, reply".into());
    }
    let send = if let Some(send) = object.get("send") {
        if !joined {
            return Err("hook cannot send realtime before a peer joined".into());
        }
        Some(crate::realtime::payload(
            send.as_str().ok_or("hook send must be a UTF-8 string")?,
            false,
        )?)
    } else {
        None
    };
    let update = if let Some(update) = object.get("update") {
        if !update.is_object() || update.get("payload").is_none() {
            return Err("hook update must be an object containing payload".into());
        }
        Some(update.to_string())
    } else {
        None
    };
    let reply = if let Some(reply) = object.get("reply") {
        let text = reply
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .ok_or("hook reply must be nonempty text")?;
        let event = StatusUpdateItem {
            payload: input.parse()?,
            ..Default::default()
        };
        if !matches!(
            event.payload.get("type").and_then(|v| v.as_str()),
            Some("message_received" | "command_received")
        ) {
            return Err("reply requires a message or command event".into());
        }
        let msg_id: u32 = event
            .payload
            .get("incoming_message_id")
            .and_then(|v| v.as_u64())
            .ok_or("missing incoming message ID")?
            .try_into()?;
        let original =
            deltachat::message::Message::load_from_db(context, MsgId::new(msg_id)).await?;
        let instance = deltachat::message::Message::load_from_db(context, id).await?;
        if original.get_chat_id() != instance.get_chat_id()
            || original.get_from_id() == deltachat::contact::ContactId::SELF
        {
            return Err("reply must target an incoming message in this app chat".into());
        }
        Some((text.to_owned(), original))
    } else {
        None
    };
    if let Some(update) = update {
        context.send_webxdc_status_update(id, &update).await?;
    }
    if let Some((text, original)) = reply {
        let mut message = deltachat::message::Message::new_text(text);
        message.set_quote(context, Some(&original)).await?;
        deltachat::chat::send_msg(context, original.get_chat_id(), &mut message).await?;
    }
    if let Some(data) = send {
        deltachat::peer_channels::send_webxdc_realtime_data(context, id, data).await?;
    }
    Ok(())
}
