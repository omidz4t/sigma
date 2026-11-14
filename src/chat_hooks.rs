//! Persistent command hooks scoped to core chat IDs, independent of webxdc.
use crate::{Result, app_hooks::Hook, cli::Args, output, shared};
use deltachat::{
    chat, contact::ContactId, context::Context, message::Message, webxdc::StatusUpdateItem,
};
use std::path::Path;

fn load(root: &Path, chat_id: u32) -> Result<Vec<Hook>> {
    let path = root.join("chat-hooks").join(format!("{chat_id}.json"));
    if !path.exists() {
        return Ok(vec![]);
    }
    let v = StatusUpdateItem {
        payload: std::fs::read_to_string(path)?.parse()?,
        ..Default::default()
    };
    v.payload
        .as_array()
        .ok_or("invalid chat hooks file")?
        .iter()
        .map(|v| Hook::parse(&v.to_string()))
        .collect()
}
fn save(root: &Path, chat_id: u32, hooks: &[Hook]) -> Result<()> {
    let dir = root.join("chat-hooks");
    std::fs::create_dir_all(&dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }
    let path = dir.join(format!("{chat_id}.json"));
    let tmp = dir.join(format!("{chat_id}.tmp"));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&tmp)?;
    use std::io::Write;
    file.write_all(
        output::encode(&output::Value::Array(
            hooks.iter().map(Hook::value).collect(),
        ))
        .as_bytes(),
    )?;
    file.sync_all()?;
    std::fs::rename(tmp, path)?;
    Ok(())
}
pub(crate) async fn manage(account: &crate::account::Account, args: &Args) -> Result<()> {
    let id: u32 = args.positional[0].parse()?;
    shared::sendable_chat(&account.context, id).await?;
    let mut hooks = load(&account.path, id)?;
    match args.command.as_str() {
        "chat-hook-add" => {
            let name = &args.positional[1];
            crate::apps::validate_name(name)?;
            if hooks.iter().any(|h| h.id == *name) {
                return Err("hook ID already exists in this chat".into());
            }
            let command = args
                .option("--command")
                .ok_or("chat-hook-add requires --command /NAME")?;
            if crate::app_hooks::command(command)
                .is_none_or(|(cmd, rest)| cmd != command || !rest.is_empty())
            {
                return Err("--command must be a single slash command".into());
            }
            hooks.push(Hook {
                id: name.clone(),
                path: crate::app_hooks::executable(&args.positional[2])?,
                event: "command_received".into(),
                match_json: Some(output::encode(&output::object(vec![(
                    "command",
                    command.into(),
                )]))),
                timeout: args.option("--hook-timeout").unwrap_or("30").parse()?,
                after_seq: 0,
            });
            save(&account.path, id, &hooks)?;
        }
        "chat-hook-remove" => {
            let old = hooks.len();
            hooks.retain(|h| h.id != args.positional[1]);
            if old == hooks.len() {
                return Err("unknown chat hook ID".into());
            }
            save(&account.path, id, &hooks)?;
        }
        _ => {}
    }
    output::record(
        "chat_hooks",
        vec![
            ("chat_id", id.into()),
            (
                "hooks",
                output::Value::Array(hooks.iter().map(Hook::value).collect()),
            ),
        ],
        &format!("Chat {id}: {} command hooks", hooks.len()),
    );
    Ok(())
}
pub(crate) async fn incoming(context: &Context, root: &Path, message: &Message) -> Result<()> {
    if message.get_from_id() == ContactId::SELF || message.is_info() {
        return Ok(());
    }
    let text = message.get_text();
    let Some((command, args)) = crate::app_hooks::command(&text) else {
        return Ok(());
    };
    let id = message.get_chat_id().to_u32();
    for hook in load(root, id)? {
        let filter = StatusUpdateItem {
            payload: hook.match_json.as_deref().unwrap_or("{}").parse()?,
            ..Default::default()
        };
        if filter.payload.get("command").and_then(|v| v.as_str()) != Some(&command) {
            continue;
        }
        let input = output::encode(&output::object(vec![
            ("type", "command_received".into()),
            ("hook_id", hook.id.clone().into()),
            ("chat_id", id.into()),
            ("incoming_message_id", message.get_id().to_u32().into()),
            ("text", text.clone().into()),
            ("command", command.clone().into()),
            ("args", args.clone().into()),
            ("message", shared::message_value(context, message).await?),
        ]));
        let result: Result<()> = async {
            let raw = crate::automation::run_hook(&hook.path, &input, hook.timeout).await?;
            if !raw.trim().is_empty() {
                let v = StatusUpdateItem {
                    payload: raw.parse()?,
                    ..Default::default()
                };
                let object = v
                    .payload
                    .as_object()
                    .ok_or("chat hook output must be an object")?;
                if object.keys().any(|k| k != "reply") {
                    return Err("chat hooks support only the reply action".into());
                }
                if let Some(reply) = object.get("reply") {
                    let text = reply
                        .as_str()
                        .filter(|s| !s.trim().is_empty())
                        .ok_or("reply must be a nonempty string")?;
                    let mut reply = Message::new_text(text.into());
                    reply.set_quote(context, Some(message)).await?;
                    chat::send_msg(context, message.get_chat_id(), &mut reply).await?;
                }
            }
            Ok(())
        }
        .await;
        output::record(
            "chat_hook_finished",
            vec![
                ("chat_id", id.into()),
                ("hook_id", hook.id.into()),
                ("incoming_message_id", message.get_id().to_u32().into()),
                ("success", result.is_ok().into()),
                (
                    "error",
                    result
                        .err()
                        .map(|e| e.to_string().into())
                        .unwrap_or(output::Value::Null),
                ),
            ],
            "Chat command hook finished",
        );
    }
    Ok(())
}
