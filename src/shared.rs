use crate::{Result, cli::Args};
use deltachat::{
    EventType,
    chat::{self, Chat, ChatId},
    constants::Chattype,
    contact::{Contact, ContactId},
    context::Context,
    message::{Message, MessageState, MsgId, Viewtype},
    qr::{Qr, check_qr},
    reaction::{self, Reaction},
    securejoin,
};

pub(crate) fn validate_reaction(raw: &str) -> Result<()> {
    if Reaction::new(raw).as_str() != raw {
        return Err("reaction must be a single token shorter than 30 UTF-8 bytes; omit it to remove your reaction".into());
    }
    Ok(())
}

pub(crate) async fn wait_for_reaction(context: &Context, id: MsgId, seconds: u64) -> Result<()> {
    let sending = async {
        loop {
            match Message::load_from_db(context, id).await?.get_state() {
                MessageState::OutDelivered | MessageState::OutMdnRcvd => return Ok(()),
                MessageState::OutFailed => {
                    return Err("reaction sending failed; inspect state before retrying".into());
                }
                _ => tokio::time::sleep(std::time::Duration::from_millis(250)).await,
            }
        }
    };
    tokio::select! {
        result = tokio::time::timeout(std::time::Duration::from_secs(seconds), sending) => {
            result.map_err(|_| "reaction send timed out; reaction remains stored and may be queued. Run listen to continue sending")?
        }
        signal = tokio::signal::ctrl_c() => {
            signal?;
            Err("reaction interrupted; reaction remains stored and may be queued. Run listen to continue sending".into())
        }
    }
}

pub(crate) async fn message_line(context: &Context, message: &Message) -> Result<String> {
    let sender = Contact::get_by_id(context, message.get_from_id()).await?;
    let reactions = reaction::get_msg_reactions(context, message.get_id()).await?;
    Ok(format!(
        "message={}\tchat={}\tsender={}\tname={:?}\taddress={:?}\ttimestamp={}\tstate={:?}\ttype={:?}\ttext={:?}\tfilename={:?}\tfile={:?}\treactions={:?}",
        message.get_id().to_u32(),
        message.get_chat_id().to_u32(),
        message.get_from_id().to_u32(),
        sender.get_name(),
        sender.get_addr(),
        message.get_timestamp(),
        message.get_state(),
        message.get_viewtype(),
        message.get_text(),
        message.get_filename(),
        message.get_file(context),
        reactions.to_string()
    ))
}

pub(crate) async fn outgoing_channel(context: &Context, id: u32) -> Result<ChatId> {
    let id = ChatId::new(id);
    let channel = Chat::load_from_db(context, id).await?;
    if channel.get_type() != Chattype::OutBroadcast {
        return Err("CHANNEL_ID must identify a broadcast channel owned by this account".into());
    }
    Ok(id)
}

pub(crate) fn save_join_qr(link: &str, output: &std::path::Path) -> Result<()> {
    let svg = deltachat::qr_code_generator::create_qr_svg(link)?;
    // Refuse existing paths: never overwrite a source file or account database.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    use std::io::Write;
    file.write_all(svg.as_bytes())?;
    Ok(())
}

pub(crate) fn text_file_message(path: &std::path::Path) -> Result<Message> {
    if !path.is_file() {
        return Err("file must be an existing local regular file".into());
    }
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read text file as UTF-8: {error}"))?;
    if text.trim().is_empty() {
        return Err("text file cannot be empty or whitespace-only".into());
    }
    Ok(Message::new_text(text))
}

pub(crate) fn attachment_message(
    context: &Context,
    path: &std::path::Path,
    caption: Option<&str>,
    viewtype: Viewtype,
) -> Result<Message> {
    let path = std::fs::canonicalize(path)?;
    if !path.is_file() {
        return Err("file must be an existing local regular file".into());
    }
    let mut message = Message::new(viewtype);
    message.set_file_and_deduplicate(context, &path, None, None)?;
    if let Some(caption) = caption {
        message.set_text(caption.to_owned());
    }
    Ok(message)
}

pub(crate) fn is_securejoin(value: &str) -> bool {
    value.starts_with("https://i.delta.chat/#")
        || value.starts_with("https://i.delta.chat#")
        || value.to_ascii_lowercase().starts_with("openpgp4fpr:")
}

pub(crate) async fn join_contact(
    context: &Context,
    invite: &str,
    timeout: Option<&str>,
) -> Result<(ContactId, ChatId)> {
    join_invitation(context, invite, timeout, "contact").await
}

pub(crate) async fn join_invitation(
    context: &Context,
    invite: &str,
    timeout: Option<&str>,
    kind: &str,
) -> Result<(ContactId, ChatId)> {
    ensure_configured(context).await?;
    let contact_id = match check_qr(context, invite).await? {
        Qr::AskVerifyContact { contact_id, .. } if kind == "contact" => contact_id,
        Qr::AskVerifyGroup { contact_id, .. } if kind == "group" => contact_id,
        Qr::AskJoinBroadcast { contact_id, .. } if kind == "channel" => contact_id,
        _ => return Err(format!("expected a {kind} secure-join invitation").into()),
    };
    let seconds = timeout.unwrap_or("120").parse::<u64>()?;
    // Subscribe before joining: core may emit progress during join_securejoin().
    let events = context.get_event_emitter();
    context.start_io().await;
    let result = async {
        let chat_id = securejoin::join_securejoin(context, invite).await?;
        loop {
            let event = events
                .recv()
                .await
                .ok_or("core event stream closed during secure join")?;
            if let EventType::SecurejoinJoinerProgress {
                contact_id: id,
                progress,
            } = event.typ
                && id == contact_id
                && progress == 1000
            {
                return Ok((contact_id, chat_id));
            }
        }
    };
    let outcome = tokio::select! {
        result = tokio::time::timeout(std::time::Duration::from_secs(seconds), result) => {
            match result {
                Ok(result) => result,
                Err(_) => Err("secure join timed out; no user message sent. Run listen to continue the handshake, then retry".into()),
            }
        }
        signal = tokio::signal::ctrl_c() => {
            match signal {
                Ok(()) => Err("secure join interrupted; no user message sent".into()),
                Err(error) => Err(error.into()),
            }
        }
    };
    context.stop_io().await;
    outcome
}

pub(crate) async fn ensure_configured(context: &Context) -> Result<()> {
    if !context.is_configured().await? {
        return Err("account is not configured; retry init".into());
    }
    Ok(())
}
pub(crate) async fn send_to_target_id(
    context: &Context,
    args: &Args,
    mut message: Message,
) -> Result<MsgId> {
    let chat = if args.option("--chat").is_some() {
        sendable_chat(context, args.positional[0].parse()?).await?
    } else if is_securejoin(&args.positional[0]) {
        join_contact(context, &args.positional[0], args.option("--timeout"))
            .await?
            .1
    } else {
        let contact = crate::aliases::resolve(context, &args.positional[0]).await?;
        Contact::get_by_id(context, contact).await?;
        ChatId::create_for_contact(context, contact).await?
    };
    let id = chat::send_msg_sync(context, chat, &mut message).await?;
    crate::output::record(
        "sent",
        vec![
            ("message_id", id.to_u32().into()),
            ("chat_id", chat.to_u32().into()),
        ],
        &format!("Sent message {}", id.to_u32()),
    );
    Ok(id)
}

pub(crate) async fn sendable_chat(context: &Context, id: u32) -> Result<ChatId> {
    let id = ChatId::new(id);
    if !Chat::load_from_db(context, id)
        .await?
        .can_send(context)
        .await?
    {
        return Err("cannot send in this chat; it may be read-only or require acceptance".into());
    }
    Ok(id)
}

pub(crate) async fn message_value(
    context: &Context,
    message: &Message,
) -> Result<crate::output::Value> {
    use crate::output::{Value, object};
    let sender = Contact::get_by_id(context, message.get_from_id()).await?;
    Ok(object(vec![
        ("message_id", message.get_id().to_u32().into()),
        ("chat_id", message.get_chat_id().to_u32().into()),
        ("sender_id", message.get_from_id().to_u32().into()),
        ("sender_name", sender.get_name().into()),
        ("sender_address", sender.get_addr().into()),
        ("timestamp", message.get_timestamp().into()),
        ("state", format!("{:?}", message.get_state()).into()),
        ("viewtype", format!("{:?}", message.get_viewtype()).into()),
        ("duration_ms", i64::from(message.get_duration()).into()),
        ("text", message.get_text().into()),
        (
            "filename",
            message
                .get_filename()
                .map(Value::from)
                .unwrap_or(Value::Null),
        ),
        (
            "file",
            message
                .get_file(context)
                .map(|p| Value::from(p.to_string_lossy().into_owned()))
                .unwrap_or(Value::Null),
        ),
        (
            "reactions",
            reaction::get_msg_reactions(context, message.get_id())
                .await?
                .to_string()
                .into(),
        ),
        (
            "parent_id",
            message
                .parent(context)
                .await?
                .map(|m| Value::from(m.get_id().to_u32()))
                .unwrap_or(Value::Null),
        ),
        ("forwarded", message.is_forwarded().into()),
        ("edited", message.is_edited().into()),
        (
            "download_state",
            format!("{:?}", message.download_state()).into(),
        ),
    ]))
}
pub(crate) async fn print_message(context: &Context, message: &Message) -> Result<()> {
    crate::output::record(
        "message",
        vec![("message", message_value(context, message).await?)],
        &message_line(context, message).await?,
    );
    Ok(())
}

pub(crate) async fn managed_chat(context: &Context, id: u32, expected: Chattype) -> Result<ChatId> {
    let id = ChatId::new(id);
    let chat = Chat::load_from_db(context, id).await?;
    if chat.get_type() != expected {
        return Err(format!("expected {expected:?} chat").into());
    }
    if !chat.is_self_in_chat(context).await? {
        return Err("this account is not in the chat".into());
    }
    Ok(id)
}
pub(crate) async fn list_members(context: &Context, id: ChatId) -> Result<()> {
    for contact_id in chat::get_chat_contacts(context, id).await? {
        let contact = Contact::get_by_id(context, contact_id).await?;
        crate::output::record(
            "member",
            vec![
                ("chat_id", id.to_u32().into()),
                ("contact_id", contact_id.to_u32().into()),
                ("name", contact.get_name().into()),
                ("address", contact.get_addr().into()),
            ],
            &format!(
                "{}\t{:?}\t{}",
                contact_id.to_u32(),
                contact.get_name(),
                contact.get_addr()
            ),
        );
    }
    Ok(())
}
pub(crate) fn private_directory(path: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
pub(crate) fn backup_passphrase(args: &Args) -> Result<Option<String>> {
    args.option("--passphrase-file")
        .map(|path| {
            let mut text = std::fs::read_to_string(path)?;
            if text.ends_with('\n') {
                text.pop();
                if text.ends_with('\r') {
                    text.pop();
                }
            }
            if text.is_empty() {
                return Err("passphrase file must not be empty".into());
            }
            Ok(text)
        })
        .transpose()
}
pub(crate) async fn download_message(
    context: &Context,
    id: MsgId,
    timeout: Option<&str>,
) -> Result<Message> {
    use deltachat::download::DownloadState;
    let message = Message::load_from_db(context, id).await?;
    if message.download_state() == DownloadState::Done {
        return Ok(message);
    }
    ensure_configured(context).await?;
    if message.download_state() == DownloadState::Undecipherable {
        return Err("message cannot be decrypted".into());
    }
    if message.download_state() != DownloadState::InProgress {
        id.download_full(context).await?;
    }
    let seconds = timeout.unwrap_or("120").parse::<u64>()?;
    context.start_io().await;
    let operation = async {
        loop {
            let message = Message::load_from_db(context, id).await?;
            match message.download_state() {
                DownloadState::Done => return Ok(message),
                DownloadState::Failure | DownloadState::Undecipherable => {
                    return Err("message download failed".into());
                }
                _ => tokio::time::sleep(std::time::Duration::from_millis(250)).await,
            }
        }
    };
    let result = tokio::select! {
        result = tokio::time::timeout(std::time::Duration::from_secs(seconds), operation) => match result {
            Ok(result) => result,
            Err(_) => Err("download timed out; run listen to continue".into()),
        },
        signal = tokio::signal::ctrl_c() => match signal { Ok(()) => Err("download interrupted; run listen to continue".into()), Err(error) => Err(error.into()) }
    };
    context.stop_io().await;
    result
}

pub(crate) async fn send_to_target(context: &Context, args: &Args, message: Message) -> Result<()> {
    send_to_target_id(context, args, message).await?;
    Ok(())
}

pub(crate) fn avatar_path(raw: &str) -> Result<String> {
    if raw.is_empty() {
        return Ok(String::new());
    }
    let path = std::fs::canonicalize(raw)?;
    if !path.is_file() {
        return Err("photo must be an existing local regular file".into());
    }
    Ok(path.to_str().ok_or("photo path must be UTF-8")?.to_owned())
}
