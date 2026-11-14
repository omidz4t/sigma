//! A local agent process bridged to one explicitly selected direct chat.
use crate::{Result, account::Account, cli::Args, help::CommandHelp, output, shared};
use deltachat::{
    EventType,
    chat::{self, ChatId},
    message::{Message, MsgId},
};
use std::path::{Path, PathBuf};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "agent-connect",
    arguments: "PROVIDER [--to TARGET]",
    description: "Bridge a local agent executable to a contact ID or saved alias such as me. Send a connection message and start a background worker. Each agent has a saved alias and automatic account-local ID, permanently bound to its first recipient. Restarts may omit --to. Use agent-list and agent-stop to manage it. Only authorized senders reach the executable: the current bot master by default, including in groups containing the owner, plus chat-scoped owner approvals. Owner-only /approve, /assign and /revoke reply commands manage permissions; /access lists approvals and /access remove CONTACT_ID removes them; agent-permissions manages them from the CLI. Files are staged into the current chat outbox. Groups use separate native sessions. Native Grok/Codex turns resume one saved provider session for this agent, retaining history across worker restarts. Existing chat history is imported once when creating that session. Grok uses headless turns and Codex uses exec with a writable outbox sandbox; custom adapters read a plain UTF-8 prompt on stdin. Stdout becomes the reply. This is a local process bridge, not the HTTP A2A protocol. The worker holds the account lock; stop it before other messaging commands.",
    options: "  --alias NAME        Agent alias (default: provider name)\n  --agent-id ID       Select an existing agent by its automatic ID\n  --to TARGET         Required for a new agent; initializes its sole bot master\n  --exec PATH         Executable adapter (default: AGENT found on PATH)\n  --new-account NAME  Create a dedicated named account before connecting\n  --name TEXT         Agent account display name\n  --profile-photo PATH  Agent account profile image\n  --invite LINK       Join this contact invitation and save --to as its alias\n  --account-qr QR     Optional provisioning QR for --new-account\n  --model MODEL       Native Grok/Codex model (default: CLI configuration)\n  --system-prompt TEXT  Starting instructions for native Grok/Codex (model is appended)\n  --system-prompt-file PATH  Read the starting instructions from a UTF-8 file\n  --allow-outbound-calls  Allow master-requested outgoing Codex audio calls (requires installed Chromium)\n  --record-calls     Record both sides of Codex calls, keep files and send to master (requires Chromium)\n  --foreground       Run here until Ctrl-C instead of starting in background\n  --hook-timeout SEC  Per-message execution limit (default: 120)\n",
    example: "agent-connect codex --new-account codex-bot --name Codex --to me --invite INVITATION",
    summary: "Bridge a local agent to one contact and keep receiving",
};

pub(crate) fn executable(name: &str, explicit: Option<&str>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return crate::app_hooks::executable(path);
    }
    let search = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&search) {
        let candidate = dir.join(name);
        if let Ok(path) = crate::app_hooks::executable(&candidate.to_string_lossy()) {
            return Ok(path);
        }
    }
    Err(format!("agent executable '{name}' was not found on PATH; use --exec PATH to an executable adapter that reads a prompt on stdin and prints its reply").into())
}

pub(crate) fn starting_prompt(model: Option<&str>, custom: Option<&str>) -> String {
    let identity = match model {
        Some(model) => format!("Model: {model}."),
        None => "Model: the provider CLI's configured default; Sigma has not supplied an explicit model ID.".into(),
    };
    match custom {
        Some(prompt) => format!("{prompt}\n\n{identity}"),
        None => identity,
    }
}

pub(crate) struct ReplySettings<'a> {
    pub(crate) native: Option<&'a str>,
    pub(crate) model: Option<&'a str>,
    pub(crate) system_prompt: Option<&'a str>,
    pub(crate) session_dir: Option<&'a Path>,
    pub(crate) actions: Option<&'a crate::agent_actions::Server>,
    pub(crate) import_history: bool,
}

pub(crate) async fn reply(
    context: &deltachat::context::Context,
    executable: &Path,
    timeout: u64,
    settings: &ReplySettings<'_>,
    message: &Message,
) -> Result<Option<MsgId>> {
    let mut environment = settings
        .actions
        .map(|a| a.environment.clone())
        .unwrap_or_default();
    environment.push((
        "SIGMA_INCOMING_MESSAGE_ID".into(),
        message.get_id().to_u32().to_string(),
    ));
    let audio = crate::agent_voice::is_audio(message);
    if audio && message.get_file(context).is_none() {
        if let Some(actions) = settings.actions {
            actions.check(context).await?;
        }
        return crate::agent_voice::unsupported(context, message).await;
    }
    let mut incoming = message.get_text();
    if let Some(actions) = settings.actions {
        incoming = format!(
            "Sender contact ID: {}. Chat ID: {}. Message ID: {}.\n{}",
            message.get_from_id().to_u32(),
            message.get_chat_id().to_u32(),
            message.get_id().to_u32(),
            incoming
        );
        if let Some(file) = message.get_file(context) {
            let target = actions.outbox.join(format!(
                "incoming-{}-{}-{}",
                message.get_id().to_u32(),
                uuid::Uuid::new_v4(),
                file.file_name().unwrap_or_default().to_string_lossy()
            ));
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let staged: Result<()> = (|| {
                let mut destination = options.open(&target)?;
                std::io::copy(&mut std::fs::File::open(file)?, &mut destination)?;
                Ok(())
            })();
            if let Err(error) = staged {
                let _ = std::fs::remove_file(&target);
                if audio {
                    actions.check(context).await?;
                    return crate::agent_voice::unsupported(context, message).await;
                }
                return Err(error);
            }
            incoming.push_str(&format!(
                "\nAttachment ({:?}): {}",
                message.get_viewtype(),
                target.display()
            ));
            if audio {
                environment.push((
                    "SIGMA_INCOMING_VOICE_PATH".into(),
                    target.to_string_lossy().into_owned(),
                ));
                environment.push((
                    "SIGMA_INCOMING_VOICE_DURATION_MS".into(),
                    message.get_duration().to_string(),
                ));
            }
        }
    }
    if audio {
        incoming.push_str("\n\nIncoming audio task: transcribe this recording yourself, then use the actual transcript as the user's request. If no usable transcript can be obtained with your available capabilities, return exactly [SIGMA_VOICE_UNSUPPORTED] so Sigma reacts with a cross. The caption above is not a transcript.\n");
        incoming.push_str(crate::agent_voice::instructions());
        // Adapters without an action server still receive a readable core attachment path.
        if settings.actions.is_none()
            && let Some(file) = message.get_file(context)
        {
            incoming.push_str(&format!("\nAudio path: {}", file.display()));
            environment.push((
                "SIGMA_INCOMING_VOICE_PATH".into(),
                file.to_string_lossy().into_owned(),
            ));
        }
    }
    let text = if let Some(provider @ ("grok" | "codex")) = settings.native {
        let dir = settings
            .session_dir
            .ok_or("native agents require a persistent session directory")?;
        let input = if crate::agent_sessions::load(dir)?.is_none() && settings.import_history {
            format!(
                "{}\n\n{}",
                crate::agent_sessions::bootstrap(context, message).await?,
                incoming
            )
        } else {
            incoming.clone()
        };
        let mut prompt = starting_prompt(settings.model, settings.system_prompt);
        prompt.push('\n');
        prompt.push_str(crate::agent_guard::instructions());
        prompt.push('\n');
        prompt.push_str(crate::agent_voice::instructions());
        if let Some(actions) = settings.actions {
            prompt.push_str("\n\n");
            prompt.push_str(&actions.instructions());
            prompt.push_str(&format!(
                "\nCurrent incoming message ID: {}.",
                message.get_id().to_u32()
            ));
        }
        crate::agent_sessions::Invocation {
            executable,
            provider,
            model: settings.model,
            prompt: &prompt,
            seconds: timeout,
            dir,
            environment: &environment,
            workspace: settings.actions.map(|a| a.outbox.as_path()),
        }
        .run(&input)
        .await?
    } else {
        crate::automation::run_process_env(executable, &[], &incoming, timeout, &environment)
            .await?
    };
    if audio && text.trim().ends_with(crate::agent_voice::UNSUPPORTED) {
        if let Some(actions) = settings.actions {
            actions.check(context).await?;
        }
        return crate::agent_voice::unsupported(context, message).await;
    }
    if text.trim().is_empty() || text.trim().ends_with(crate::agent_actions::NO_REPLY) {
        return Ok(None);
    }
    if let Some(actions) = settings.actions {
        actions.check(context).await?;
    }
    let chat = shared::sendable_chat(context, message.get_chat_id().to_u32()).await?;
    let mut response = Message::new_text(text);
    response.set_quote(context, Some(message)).await?;
    let id = chat::send_msg(context, chat, &mut response).await?;
    output::record(
        "agent_reply_queued",
        vec![
            ("message_id", id.to_u32().into()),
            ("parent_id", message.get_id().to_u32().into()),
            ("chat_id", chat.to_u32().into()),
        ],
        &format!("Agent reply {} queued", id.to_u32()),
    );
    Ok(Some(id))
}

pub(crate) async fn handle(account: &Account, args: &Args) -> Result<()> {
    let prompt_file = args
        .option("--system-prompt-file")
        .map(std::fs::read_to_string)
        .transpose()?;
    let system_prompt = args.option("--system-prompt").or(prompt_file.as_deref());
    let context = &account.context;
    shared::ensure_configured(context).await?;
    // Validate everything before creating a chat or sending a notification.
    let agent = &args.positional[0];
    let executable = executable(agent, args.option("--exec"))?;
    let saved = crate::agent_registry::selected(&account.path, args)?;
    let contact = if let Some(saved) = &saved {
        let dir = crate::agent_registry::directory(&account.path, &saved.alias)?;
        if args.option("--to").is_none() && crate::agent_master::stored(&dir)?.is_some() {
            deltachat::contact::ContactId::new(saved.contact_id)
        } else {
            saved.recipient(context, args).await?
        }
    } else {
        let target = args.option("--to").ok_or(
            "a new agent requires --to TARGET; choose --alias NAME to create a separate identity",
        )?;
        if let Some(invite) = args.option("--invite") {
            crate::aliases::ensure_available(context, target)?;
            let (contact, _) = shared::join_contact(context, invite, Some("120")).await?;
            crate::aliases::save(context, target, contact).await?;
            contact
        } else if shared::is_securejoin(target) {
            shared::join_contact(context, target, Some("120")).await?.0
        } else {
            crate::aliases::resolve(context, target).await?
        }
    };
    if contact.to_u32() <= 9 {
        return Err(
            "bridge target must be a regular contact, not a special contact or self".into(),
        );
    }
    let timeout = args.option("--hook-timeout").unwrap_or("120").parse()?;
    let chat = if let Some(saved) = &saved {
        ChatId::new(saved.chat_id)
    } else {
        ChatId::create_for_contact(context, contact).await?
    };
    let sendable = if let Some(saved) = &saved {
        crate::agent_master::current(
            context,
            &crate::agent_registry::directory(&account.path, &saved.alias)?,
            saved,
        )
        .await?
        .chat
    } else {
        chat
    };
    shared::sendable_chat(context, sendable.to_u32()).await?;
    let identity = if let Some(saved) = saved {
        if crate::agent_master::stored(&crate::agent_registry::directory(
            &account.path,
            &saved.alias,
        )?)?
        .is_none()
        {
            saved.verify(context, contact, chat).await?;
        }
        saved
    } else {
        crate::agent_registry::Agent::create(&account.path, args, context, contact, chat).await?
    };
    if args.option("--name").is_some()
        || args.option("--profile-photo").is_some()
        || args.option("--profile-photo-address").is_some()
    {
        crate::handlers::init::configure_profile(context, args, &account.path).await?;
    }
    let session_dir = crate::agent_registry::directory(&account.path, &identity.alias)?;
    let session_dir = std::fs::canonicalize(&session_dir)?;
    crate::agent_models::initialize(&session_dir, args.option("--model"))?;
    let adapter_marker = session_dir.join("custom-adapter");
    if args.option("--exec").is_some() {
        std::fs::write(&adapter_marker, "custom adapter\n")?;
    } else if adapter_marker.exists() {
        std::fs::remove_file(adapter_marker)?;
    }
    let initial_master = crate::agent_master::current(context, &session_dir, &identity).await?;
    if args.option("--record-calls").is_some() {
        crate::call_browser::executable()?;
    }
    let events = context.get_event_emitter();
    let mut notification = Message::new_text(format!(
        "Connected to {} ({}; ID {}) through Sigma. Send a text message here to talk to the agent. The bridge stays online until its Sigma agent worker is stopped.",
        identity.alias, agent, identity.id
    ));
    // Queue first, then enable networking with the event subscription already in place.
    chat::send_msg(context, initial_master.chat, &mut notification).await?;
    let mut runtime = crate::agents::register_with_context(
        &account.path,
        args,
        chat.to_u32(),
        Some(context.clone()),
    )
    .await?;
    context.start_io().await;
    output::record(
        "agent_connected",
        vec![
            ("agent", agent.as_str().into()),
            ("alias", identity.alias.as_str().into()),
            ("id", identity.id.into()),
            (
                "model",
                args.option("--model")
                    .map(output::Value::from)
                    .unwrap_or(output::Value::Null),
            ),
            ("contact_id", contact.to_u32().into()),
            ("chat_id", chat.to_u32().into()),
            ("notification_id", notification.get_id().to_u32().into()),
        ],
        &format!("Bridge to {agent} running; connection message queued. Press Ctrl-C to stop."),
    );
    let mut handled = std::collections::HashSet::new();
    let mut active_call: Option<crate::agent_calls::Active> = None;
    let (_outbound_permission, mut outbound_requests) =
        if args.option("--allow-outbound-calls").is_some() {
            let (permission, rx) = crate::agent_outbound::register(&session_dir)?;
            (Some(permission), rx)
        } else {
            let (_, rx) = tokio::sync::mpsc::channel(1);
            (None, rx)
        };
    let mut outbound_call: Option<crate::agent_outbound::Active> = None;

    let result: Result<()> = async {
        loop {
            tokio::select! {
                _ = runtime.stop.changed() => { break; }
                signal = tokio::signal::ctrl_c() => { signal?; break; }
                Some(request) = outbound_requests.recv(), if _outbound_permission.is_some() => {
                    if outbound_call.as_ref().is_some_and(|c|c.task.is_finished()){outbound_call.take();}
                    if active_call.as_ref().is_some_and(|c|c.task.is_finished()){active_call.take();}
                    if outbound_call.is_some() || active_call.is_some() {
                        let _=request.ticket.update("failed",None,Some("another call is active"));
                        let mut notice=Message::new_text("Outbound call was not placed: another call is active.".into());
                        chat::send_msg(context,request.scope.owner_chat,&mut notice).await?;
                    } else {
                        let model=crate::agent_models::selected(&session_dir,args.option("--model"))?;
                        outbound_call=Some(crate::agent_outbound::start(context.clone(),executable.clone(),request,model,args.option("--record-calls").is_some()));
                    }
                }
                event = events.recv() => {
                    let Some(event) = event else { break; };
                    if let EventType::IncomingMsg { msg_id, .. } = event.typ {
                        let mut message = Message::load_from_db(context, msg_id).await?;
                        if message.get_viewtype()==deltachat::message::Viewtype::Call {continue;}
                        let master = crate::agent_master::current(context, &session_dir, &identity).await?;
                        let contact = master.contact;
                        let chat = master.chat;
                        if !handled.insert(msg_id) { continue; }
                        if active_call.as_ref().is_some_and(|call|call.task.is_finished()){active_call.take();}
                        let text=message.get_text();
                        if text.split_whitespace().next()==Some("/call") {
                            if message.get_from_id()!=contact {continue;}
                            if text.trim()=="/call stop" {while let Ok(request)=outbound_requests.try_recv(){request.ticket.cancel();let _=request.ticket.update("cancelled",None,None);} if let Some(call)=&outbound_call {call.stop();}}
                            if text.trim()=="/call stop" && let Some(mut call)=active_call.take(){let _=context.end_call(call.id).await;let _=tokio::time::timeout(std::time::Duration::from_secs(10), &mut call.task).await;}
                            let mut capability=crate::agent_calls::description(agent,args.option("--exec").is_none()).to_owned();
                            capability.push_str(if args.option("--allow-outbound-calls").is_some(){" Outbound calls are enabled: ask me privately to call a contact and supply the call brief."}else{" Outbound calls are disabled; the trusted terminal must start this bridge with --allow-outbound-calls."});
                            capability.push_str(if args.option("--record-calls").is_some(){" Recording is enabled: both sides are saved locally and sent to the master."}else{" Recording is disabled."});
                            let mut response=Message::new_text(capability);response.set_quote(context,Some(&message)).await?;chat::send_msg(context,message.get_chat_id(),&mut response).await?;continue;
                        }

                        if let Some(text) = crate::agent_guard::command(context, &session_dir, contact, chat, &message).await? {
                            let approved = text.starts_with("Approved contact ");
                            let mut response = Message::new_text(text);
                            response.set_quote(context, Some(&message)).await?;
                            chat::send_msg(context, message.get_chat_id(), &mut response).await?;
                            if !approved { continue; }
                            let Some(parent) = message.parent(context).await? else { continue; };
                            message = parent;
                            handled.insert(message.get_id());
                        }
                        // Reserved owner commands are never delegated to the model, even for approved guests.
                        if crate::agent_guard::is_command(&message.get_text()) { continue; }
                        if !crate::agent_guard::allowed(context, &session_dir, contact, chat, &message).await?
                            || (message.get_text().trim().is_empty() && message.get_file(context).is_none()) { continue; }
                        // Outbound voice uses a separate session, so the master can still
                        // issue natural-language management requests in the private chat.
                        if active_call.as_ref().is_some_and(|call|!call.task.is_finished())
                            || (outbound_call.as_ref().is_some_and(|call|!call.task.is_finished())
                                && (message.get_from_id()!=contact || message.get_chat_id()!=chat)){continue;}
                        let turn_dir = if message.get_chat_id().to_u32() == identity.chat_id { session_dir.clone() }
                            else { session_dir.join(format!("chat-{}", message.get_chat_id().to_u32())) };
                        let actions = crate::agent_actions::Server::start_for_sender(context, message.get_chat_id(), &turn_dir, message.get_from_id() == contact, Some(crate::agent_guard::Scope { dir: session_dir.clone(), owner: contact, owner_chat: chat, chat: message.get_chat_id(), sender: message.get_from_id() })).await?;
                        let selected_model = crate::agent_models::selected(&session_dir, args.option("--model"))?;
                        let recording_prompt = format!("{}\nCall recording is {} for this worker. When enabled, Sigma automatically saves both sides and sends the recording to the authorized bot master's private chat after hang-up. The call agent must announce recording. This setting is controlled by the terminal --record-calls flag, not by a tool in this chat.", system_prompt.unwrap_or(""), if args.option("--record-calls").is_some(){"enabled"}else{"disabled"});
                        let settings = ReplySettings {
                            native: if args.option("--exec").is_none() { Some(agent.as_str()) } else { None },
                            model: selected_model.as_deref(), system_prompt:Some(&recording_prompt),
                            session_dir: Some(&turn_dir), actions: Some(&actions), import_history: message.get_chat_id().to_u32() == identity.chat_id,
                        };
                        tokio::select! {
                            result = reply(context, &executable, timeout, &settings, &message) => {
                                if let Err(error) = result {
                                    output::error(&format!("agent {agent} failed for message {}: {error}", msg_id.to_u32()));
                                }
                            }
                            _ = runtime.stop.changed() => { break; }
                            signal = tokio::signal::ctrl_c() => { signal?; break; }
                        }
                    } else if let EventType::IncomingCall {msg_id,place_call_info,has_video,..}=event.typ {
                        let message=Message::load_from_db(context,msg_id).await?;
                        let master=crate::agent_master::current(context,&session_dir,&identity).await?;
                        if message.get_from_id()!=master.contact || message.get_chat_id()!=master.chat || !crate::agent_guard::sender_allowed(context,&session_dir,master.contact,master.chat,message.get_chat_id(),message.get_from_id()).await.unwrap_or(false) {let _=context.end_call(msg_id).await;continue;}
                        if active_call.as_ref().is_some_and(|call|call.task.is_finished()){active_call.take();}
                        if !crate::agent_calls::supported(agent,args.option("--exec").is_none()) || crate::agent_calls::validate_offer(&place_call_info,has_video).is_err() || active_call.is_some() || outbound_call.as_ref().is_some_and(|c|!c.task.is_finished()){
                            let _=context.end_call(msg_id).await;let mut notice=Message::new_text(if has_video{"Only audio calls are supported; please start an audio call.".into()}else{crate::agent_calls::description(agent,args.option("--exec").is_none()).into()});let _=chat::send_msg(context,master.chat,&mut notice).await;continue;
                        }
                        let dir=if master.chat.to_u32()==identity.chat_id{session_dir.clone()}else{session_dir.join(format!("chat-{}",master.chat.to_u32()))};
                        std::fs::create_dir_all(&dir)?;
                        let scope=crate::agent_guard::Scope{dir:session_dir.clone(),owner:master.contact,owner_chat:master.chat,chat:master.chat,sender:master.contact};
                        let model=crate::agent_models::selected(&session_dir,args.option("--model"))?;
                        active_call=Some(crate::agent_calls::start(context.clone(),executable.clone(),crate::agent_calls::Config{dir,scope,id:msg_id,offer:place_call_info,model,custom:system_prompt.map(str::to_owned),record:args.option("--record-calls").is_some()}));
                    } else if let EventType::CallEnded {msg_id,..}=event.typ {
                        if active_call.as_ref().is_some_and(|call|call.id==msg_id && call.task.is_finished()){active_call.take();}
                    } else if let EventType::MsgDelivered { msg_id, chat_id } = event.typ {
                        output::record("agent_message_submitted", vec![("message_id", msg_id.to_u32().into()), ("chat_id", chat_id.to_u32().into())], &format!("Agent message {} submitted", msg_id.to_u32()));
                    } else if let EventType::MsgFailed { msg_id, .. } = event.typ {
                        output::error(&format!("agent message {} failed to send", msg_id.to_u32()));
                    }
                }
            }
        }
        Ok(())
    }.await;
    if let Some(mut call) = active_call.take() {
        let _ = context.end_call(call.id).await;
        let _ = tokio::time::timeout(std::time::Duration::from_secs(10), &mut call.task).await;
    }
    if let Some(mut call) = outbound_call {
        call.stop();
        let _ = tokio::time::timeout(std::time::Duration::from_secs(10), &mut call.task).await;
    }
    context.stop_io().await;
    result
}
