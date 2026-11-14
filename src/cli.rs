use crate::{
    Result,
    shared::{is_securejoin, validate_reaction},
};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct Args {
    pub(crate) command: String,
    pub(crate) positional: Vec<String>,
    pub(crate) options: BTreeMap<String, String>,
}
impl Args {
    pub(crate) fn parse(input: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut args = input.into_iter();
        let mut positional = Vec::new();
        let mut options = BTreeMap::new();
        let mut literal = false;
        while let Some(arg) = args.next() {
            if !literal && arg == "--" {
                literal = true;
            } else if !literal && matches!(arg.as_str(), "--help" | "-h" | "--version" | "-V") {
                let target = if matches!(arg.as_str(), "--help" | "-h") {
                    positional.into_iter().take(1).collect()
                } else {
                    vec![]
                };
                return Ok(Self {
                    command: arg,
                    positional: target,
                    options,
                });
            } else if !literal && arg.starts_with('-') {
                let (key, inline) = arg
                    .split_once('=')
                    .map_or((arg.as_str(), None), |(k, v)| (k, Some(v.to_owned())));
                if matches!(
                    key,
                    "--foreground"
                        | "--allow-outbound-calls"
                        | "--record-calls"
                        | "--no-hook"
                        | "--file"
                        | "--as-text"
                        | "--mark-read"
                        | "--chat"
                        | "--json"
                        | "--for-all"
                ) {
                    if inline.is_some() {
                        return Err(format!("{key} does not take a value").into());
                    }
                    if options.insert(key.to_owned(), String::new()).is_some() {
                        return Err(format!("duplicate option: {key}").into());
                    }
                    continue;
                }
                if !matches!(
                    key,
                    "--data-path"
                        | "--account"
                        | "--new-account"
                        | "--invite"
                        | "--profile-photo"
                        | "--request-id"
                        | "--alias"
                        | "--agent-id"
                        | "--to"
                        | "--exec"
                        | "--model"
                        | "--system-prompt"
                        | "--system-prompt-file"
                        | "--app"
                        | "--name"
                        | "--profile-photo-address"
                        | "--account-qr"
                        | "--timeout"
                        | "--caption"
                        | "--duration"
                        | "--output"
                        | "--chat-id"
                        | "--type"
                        | "--command"
                        | "--event"
                        | "--match"
                        | "--on-event"
                        | "--on-message"
                        | "--auto-reply"
                        | "--hook-timeout"
                        | "--passphrase-file"
                        | "--after"
                        | "--send"
                        | "--send-file"
                ) {
                    return Err(format!("unknown option: {key}").into());
                }
                let value = inline
                    .or_else(|| args.next())
                    .ok_or_else(|| format!("missing value for {key}"))?;
                if value.is_empty() || value.starts_with("--") {
                    return Err(format!("missing value for {key}").into());
                }
                if options.insert(key.to_owned(), value).is_some() {
                    return Err(format!("duplicate option: {key}").into());
                }
            } else {
                positional.push(arg);
            }
        }
        let command = if positional.is_empty() {
            "--help".into()
        } else {
            positional.remove(0)
        };
        Ok(Self {
            command,
            positional,
            options,
        })
    }
    pub(crate) fn validate(&self) -> Result<()> {
        let count = match self.command.as_str() {
            "agent-connect" | "agent-stop" | "agent-models" => 1,
            "agent-model" | "agent-master" | "agent-calls" => self.positional.len(),
            "agent-call-cancel" => 2,
            "agent-list" | "account-list" => 0,
            "agent-act" => self.positional.len(),
            "agent-permissions" => self.positional.len(),
            "account-create" | "account-default" | "account-delete" => 1,
            "app-list" => 0,
            "app-hook-add" | "chat-hook-add" => 3,
            "chat-hook-list" => 1,
            "chat-hook-remove" => 2,
            "app-hook-list" => 1,
            "app-hook-remove" => 2,
            "set-profile-photo" => 1,
            "app-info" | "app-start" | "app-stop" | "app-events" | "app-run" => 1,
            "app-bind" | "app-send" | "app-update" => 2,
            "group-members"
            | "channel-members"
            | "leave-group"
            | "leave-channel"
            | "group-link"
            | "join-group"
            | "join-channel"
            | "search-messages"
            | "delete-message"
            | "backup"
            | "restore"
            | "webxdc-updates"
            | "listen-webxdc-updates"
            | "listen-webxdc-realtime" => 1,
            "remove-from-group"
            | "remove-from-channel"
            | "rename-group"
            | "rename-channel"
            | "set-channel-description"
            | "set-group-photo"
            | "set-channel-photo"
            | "invite-to-group"
            | "edit-message"
            | "forward-message"
            | "download-file"
            | "export-file"
            | "send-webxdc-update"
            | "send-webxdc-realtime" => 2,

            "react" if self.positional.len() == 1 => 1,
            "create-contact" if self.positional.len() == 2 => 2,
            "react" => 2,
            "init" | "status" | "contacts" | "chats" | "listen" | "securejoin-id" | "join-link"
            | "give-qr" | "receive" => 0,
            "add-contact" | "messages" | "read-messages" | "create-contact" | "create-channel"
            | "create-group" | "channel-link" | "reactions" => 1,
            "add-to-group"
            | "reply"
            | "send-message"
            | "send-file"
            | "send-voice"
            | "send-photo"
            | "send-webxdc"
            | "invite-to-channel"
            | "send-channel-message" => 2,
            _ => return Err(format!("unknown command: {} (see --help)", self.command).into()),
        };
        if self.positional.len() != count {
            return Err(format!("{} expects {count} argument(s)", self.command).into());
        }
        for key in self.options.keys() {
            let allowed = matches!(key.as_str(), "--data-path" | "--account" | "--json")
                || (self.command == "agent-act"
                    && matches!(key.as_str(), "--caption" | "--duration" | "--request-id"))
                || (self.command == "agent-connect"
                    && matches!(
                        key.as_str(),
                        "--to"
                            | "--alias"
                            | "--agent-id"
                            | "--new-account"
                            | "--invite"
                            | "--name"
                            | "--profile-photo"
                            | "--profile-photo-address"
                            | "--account-qr"
                            | "--exec"
                            | "--model"
                            | "--system-prompt"
                            | "--system-prompt-file"
                            | "--foreground"
                            | "--allow-outbound-calls"
                            | "--record-calls"
                            | "--hook-timeout"
                    ))
                || (self.command == "send-webxdc" && key == "--app")
                || (self.command == "chat-hook-add"
                    && matches!(key.as_str(), "--command" | "--hook-timeout"))
                || (self.command == "app-hook-add"
                    && matches!(
                        key.as_str(),
                        "--type" | "--command" | "--event" | "--match" | "--hook-timeout"
                    ))
                || (self.command == "app-send" && key == "--file")
                || (self.command == "app-events" && key == "--after")
                || (matches!(self.command.as_str(), "app-start" | "app-run")
                    && matches!(key.as_str(), "--on-event" | "--hook-timeout" | "--no-hook"))
                || (matches!(
                    self.command.as_str(),
                    "send-message" | "send-file" | "send-voice" | "send-photo" | "send-webxdc"
                ) && key == "--chat")
                || (matches!(
                    self.command.as_str(),
                    "webxdc-updates" | "listen-webxdc-updates"
                ) && key == "--after")
                || (self.command == "send-voice" && key == "--duration")
                || (self.command == "send-webxdc-realtime" && key == "--file")
                || (self.command == "listen-webxdc-realtime"
                    && matches!(key.as_str(), "--send" | "--send-file"))
                || (self.command == "delete-message" && key == "--for-all")
                || (self.command == "search-messages" && key == "--chat-id")
                || (self.command == "listen"
                    && matches!(
                        key.as_str(),
                        "--on-message" | "--auto-reply" | "--hook-timeout"
                    ))
                || (matches!(self.command.as_str(), "backup" | "restore")
                    && key == "--passphrase-file")
                || (self.command == "read-messages" && key == "--mark-read")
                || (self.command == "give-qr" && key == "--output")
                || (matches!(self.command.as_str(), "init" | "account-create")
                    && matches!(
                        key.as_str(),
                        "--name" | "--profile-photo" | "--profile-photo-address" | "--account-qr"
                    ))
                || (self.command == "send-file" && key == "--as-text")
                || (matches!(
                    self.command.as_str(),
                    "send-file" | "send-voice" | "send-photo" | "send-webxdc"
                ) && key == "--caption")
                || (key == "--timeout"
                    && matches!(
                        self.command.as_str(),
                        "send-webxdc-realtime"
                            | "listen-webxdc-realtime"
                            | "join-group"
                            | "join-channel"
                            | "download-file"
                            | "create-contact"
                            | "send-message"
                            | "send-file"
                            | "send-voice"
                            | "send-photo"
                            | "send-webxdc"
                            | "receive"
                            | "react"
                    ))
                || (self.command == "add-contact" && key == "--name");
            if !allowed {
                return Err(format!("{key} is not valid for {}", self.command).into());
            }
        }
        if self.command == "agent-act" {
            if self.option("--account").is_some() || self.option("--data-path").is_some() {
                return Err("agent-act is scoped by its capability, not account selectors".into());
            }
            if let Some(id) = self.option("--request-id") {
                uuid::Uuid::parse_str(id).map_err(|_| "--request-id must be a UUID")?;
            }
            crate::agent_actions::cli_actions(self)?;
        }
        if self.option("--account").is_some() && self.option("--data-path").is_some() {
            return Err("--account and --data-path are mutually exclusive".into());
        }
        if let Some(name) = self.option("--account") {
            crate::aliases::validate(name)?;
        }
        if self.command.starts_with("account-") {
            if self.option("--account").is_some() || self.option("--data-path").is_some() {
                return Err("account management uses named accounts under HOME; do not combine with --account or --data-path".into());
            }
            if self.command != "account-list" {
                crate::aliases::validate(&self.positional[0])?;
            }
        }
        if self.option("--profile-photo").is_some()
            && self.option("--profile-photo-address").is_some()
        {
            return Err("use only one profile photo option".into());
        }
        if let Some(photo) = self
            .option("--profile-photo")
            .or_else(|| self.option("--profile-photo-address"))
            && !std::path::Path::new(photo).is_file()
        {
            return Err("profile photo must be an existing local file".into());
        }
        if self
            .option("--name")
            .is_some_and(|name| name.trim().is_empty())
        {
            return Err("name cannot be blank".into());
        }
        if matches!(
            self.command.as_str(),
            "agent-model" | "agent-models" | "agent-master"
        ) {
            if self.positional.is_empty() || self.positional.len() > 2 {
                return Err("expected agent-model ID_OR_ALIAS [MODEL_ID|default]".into());
            }
            if self.positional[0].parse::<u32>().is_ok() {
                positive_id(&self.positional[0])?;
            } else {
                crate::aliases::validate(&self.positional[0])?;
            }
            if let Some(model) = self.positional.get(1)
                && (model.len() > 128
                    || model.is_empty()
                    || !model
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"/._:-".contains(&b)))
            {
                return Err("invalid model ID".into());
            }
        }
        if self.command == "agent-calls" {
            if !(1..=2).contains(&self.positional.len()) {
                return Err("agent-calls expects ID_OR_ALIAS [COUNT]".into());
            }
            if let Some(count) = self.positional.get(1)
                && !count.parse::<usize>().is_ok_and(|n| (1..=100).contains(&n))
            {
                return Err("call count must be 1..100".into());
            }
        }
        if self.command == "agent-call-cancel" {
            let id = &self.positional[1];
            if id != "all"
                && !id.parse::<u32>().is_ok_and(|n| n > 9)
                && uuid::Uuid::parse_str(id).is_err()
            {
                return Err("expected call ID, request UUID or all".into());
            }
        }
        if self.command == "agent-master" && self.positional.len() == 2 {
            positive_id(&self.positional[1])?;
        }
        if self.command == "agent-permissions" {
            let valid = (self.positional.len() == 2 && self.positional[1] == "list")
                || (self.positional.len() == 4
                    && matches!(self.positional[1].as_str(), "approve" | "assign" | "revoke"));
            if !valid {
                return Err("expected agent-permissions ID_OR_ALIAS list | approve/assign/revoke CHAT_ID CONTACT_ID".into());
            }
            if self.positional[0].parse::<u32>().is_ok() {
                positive_id(&self.positional[0])?;
            } else {
                crate::aliases::validate(&self.positional[0])?;
            }
            if self.positional.len() == 4 {
                positive_id(&self.positional[2])?;
                positive_id(&self.positional[3])?;
            }
        }
        if self.command == "agent-stop" {
            if self.positional[0].parse::<u32>().is_ok() {
                positive_id(&self.positional[0])?;
            } else {
                crate::aliases::validate(&self.positional[0])?;
            }
        }
        if self.command == "agent-connect" {
            if (self.option("--allow-outbound-calls").is_some()
                || self.option("--record-calls").is_some())
                && (self.positional[0] != "codex" || self.option("--exec").is_some())
            {
                return Err(
                    "--allow-outbound-calls and --record-calls require native Codex".into(),
                );
            }
            crate::aliases::validate(&self.positional[0])?;
            if let Some(alias) = self.option("--alias") {
                crate::aliases::validate(alias)?;
            }
            if let Some(id) = self.option("--agent-id") {
                positive_id(id)?;
            }
            if self.option("--agent-id").is_some()
                && (self.option("--alias").is_some() || self.option("--new-account").is_some())
            {
                return Err("--agent-id cannot be combined with --alias or --new-account".into());
            }
            for option in ["--model", "--system-prompt", "--system-prompt-file"] {
                if let Some(value) = self.option(option) {
                    if !matches!(self.positional[0].as_str(), "grok" | "codex")
                        || self.option("--exec").is_some()
                    {
                        return Err(format!("{option} requires native Grok or Codex without --exec; configure custom adapters directly").into());
                    }
                    if value.trim().is_empty() {
                        return Err(format!("{option} cannot be blank").into());
                    }
                }
            }
            if self.option("--system-prompt").is_some()
                && self.option("--system-prompt-file").is_some()
            {
                return Err(
                    "--system-prompt and --system-prompt-file are mutually exclusive".into(),
                );
            }
            if let Some(path) = self.option("--system-prompt-file") {
                let prompt = std::fs::read_to_string(path)?;
                if prompt.trim().is_empty() {
                    return Err("system prompt file cannot be blank".into());
                }
            }
            if self.option("--account-qr").is_some() && self.option("--new-account").is_none() {
                return Err("agent-connect --account-qr requires --new-account".into());
            }
            if let Some(name) = self.option("--new-account") {
                crate::aliases::validate(name)?;
                if name == "legacy" {
                    return Err("legacy is reserved for the original account".into());
                }
                if self.option("--account").is_some() || self.option("--data-path").is_some() {
                    return Err(
                        "--new-account cannot be combined with --account or --data-path".into(),
                    );
                }
            }
            if self.option("--new-account").is_some()
                && self.option("--invite").is_none()
                && !self.option("--to").is_some_and(is_securejoin)
            {
                return Err(
                    "a new account needs --invite INVITATION with --to ALIAS, or --to INVITATION"
                        .into(),
                );
            }
            if let Some(invite) = self.option("--invite") {
                crate::aliases::validate(self.option("--to").unwrap_or(""))?;
                if !is_securejoin(invite) {
                    return Err("--invite must be a full contact invitation".into());
                }
            }
            if let Some(target) = self.option("--to") {
                if is_securejoin(target) {
                    if self.option("--invite").is_some() {
                        return Err("use --to INVITATION or --invite, not both".into());
                    }
                } else if target.parse::<u32>().is_ok() {
                    positive_id(target)?;
                } else {
                    crate::aliases::validate(target)?;
                }
            }
        }
        let id_first = matches!(
            self.command.as_str(),
            "send-webxdc-realtime"
                | "listen-webxdc-realtime"
                | "webxdc-updates"
                | "listen-webxdc-updates"
                | "send-webxdc-update"
                | "group-members"
                | "channel-members"
                | "leave-group"
                | "leave-channel"
                | "group-link"
                | "remove-from-group"
                | "remove-from-channel"
                | "rename-group"
                | "rename-channel"
                | "set-channel-description"
                | "set-group-photo"
                | "set-channel-photo"
                | "invite-to-group"
                | "delete-message"
                | "edit-message"
                | "forward-message"
                | "download-file"
                | "export-file"
        );
        if id_first {
            positive_id(&self.positional[0])?;
        }
        if matches!(
            self.command.as_str(),
            "remove-from-group" | "remove-from-channel" | "invite-to-group" | "forward-message"
        ) {
            positive_id(&self.positional[1])?;
        }
        if self.option("--send").is_some() && self.option("--send-file").is_some() {
            return Err("--send and --send-file are mutually exclusive".into());
        }
        if let Some(duration) = self.option("--duration")
            && duration
                .parse::<i32>()
                .ok()
                .filter(|value| *value >= 0)
                .is_none()
        {
            return Err("--duration must be milliseconds between 0 and 2147483647".into());
        }
        if self.command.starts_with("app-") && self.command != "app-list" {
            crate::apps::validate_name(&self.positional[0])?;
        }
        if matches!(self.command.as_str(), "app-hook-add" | "app-hook-remove") {
            crate::apps::validate_name(&self.positional[1])?;
        }
        if self.command.starts_with("chat-hook-") {
            self.positional[0]
                .parse::<u32>()
                .ok()
                .filter(|id| *id > 9)
                .ok_or("CHAT_ID must be a valid non-special chat ID")?;
            if self.command != "chat-hook-list" {
                crate::apps::validate_name(&self.positional[1])?;
            }
            if self.command == "chat-hook-add" {
                let name = self
                    .option("--command")
                    .ok_or("chat-hook-add requires --command /NAME")?;
                if crate::app_hooks::command(name)
                    .is_none_or(|(cmd, rest)| cmd != name || !rest.is_empty())
                {
                    return Err("--command must be a single slash command".into());
                }
            }
        }
        if self.command == "app-hook-add" {
            crate::app_hooks::selectors(self)?;
        }
        if self.command == "app-bind" {
            positive_id(&self.positional[1])?;
        }
        if let Some(name) = self.option("--app") {
            crate::apps::validate_name(name)?;
        }
        if self.command == "app-events" {
            if self
                .option("--after")
                .is_some_and(|s| s.parse::<i64>().ok().filter(|n| *n >= 0).is_none())
            {
                return Err("--after must be a nonnegative event sequence".into());
            }
        } else if let Some(serial) = self.option("--after") {
            serial
                .parse::<u32>()
                .map_err(|_| "--after must be an unsigned 32-bit integer")?;
        }
        if let Some(id) = self.option("--chat-id") {
            positive_id(id)?;
        }
        if let Some(value) = self.option("--hook-timeout")
            && value.parse::<u64>().ok().filter(|n| *n > 0).is_none()
        {
            return Err("--hook-timeout must be a positive integer".into());
        }
        if self.option("--on-event").is_some() && self.option("--no-hook").is_some() {
            return Err("--on-event and --no-hook are mutually exclusive".into());
        }
        if self.option("--no-hook").is_some() && self.option("--hook-timeout").is_some() {
            return Err("--hook-timeout cannot be used with --no-hook".into());
        }
        if self.option("--on-message").is_some() && self.option("--auto-reply").is_some() {
            return Err("--on-message and --auto-reply are mutually exclusive".into());
        }
        if self.option("--hook-timeout").is_some()
            && self.option("--on-message").is_none()
            && self.option("--on-event").is_none()
            && self.command != "app-hook-add"
            && self.command != "chat-hook-add"
            && self.command != "agent-connect"
        {
            return Err("--hook-timeout requires --on-message or --on-event".into());
        }
        if self
            .option("--auto-reply")
            .is_some_and(|text| text.trim().is_empty())
        {
            return Err("auto reply must not be empty".into());
        }
        if matches!(
            self.command.as_str(),
            "rename-group" | "rename-channel" | "edit-message"
        ) && self.positional[1].trim().is_empty()
        {
            return Err("name/text cannot be empty".into());
        }
        if self.command == "search-messages" && self.positional[0].trim().is_empty() {
            return Err("query cannot be empty".into());
        }
        if self.command == "restore" && !std::path::Path::new(&self.positional[0]).is_file() {
            return Err("backup file does not exist".into());
        }
        if self.command == "react" {
            let raw = self.positional.get(1).map(String::as_str).unwrap_or("");
            validate_reaction(raw)?;
        }
        if self.command == "create-contact" && !is_securejoin(self.positional.last().unwrap()) {
            return Err("SECUREJOIN_ID must be a full secure-join invitation link".into());
        }
        if self.command == "create-contact" && self.positional.len() == 2 {
            crate::aliases::validate(&self.positional[0])?;
        }
        if matches!(
            self.command.as_str(),
            "send-message" | "send-file" | "send-voice" | "send-photo" | "send-webxdc"
        ) && self.option("--chat").is_none()
            && !is_securejoin(&self.positional[0])
            && self.positional[0].parse::<u32>().is_err()
        {
            crate::aliases::validate(&self.positional[0])?;
        }
        if let Some(timeout) = self.option("--timeout")
            && timeout.parse::<u64>().ok().filter(|n| *n > 0).is_none()
        {
            return Err("--timeout must be a positive integer in seconds".into());
        }
        if matches!(
            self.command.as_str(),
            "reply"
                | "react"
                | "reactions"
                | "messages"
                | "read-messages"
                | "channel-link"
                | "add-to-group"
                | "invite-to-channel"
                | "send-channel-message"
        ) || (matches!(
            self.command.as_str(),
            "send-message" | "send-file" | "send-voice" | "send-photo" | "send-webxdc"
        ) && ((!is_securejoin(&self.positional[0])
            && self.positional[0].parse::<u32>().is_ok())
            || self.option("--chat").is_some()))
        {
            let id: u32 = self.positional[0]
                .parse()
                .map_err(|_| "ID must be a positive integer")?;
            if id == 0 {
                return Err("ID must be a positive integer".into());
            }
        }
        if matches!(
            self.command.as_str(),
            "reply" | "send-message" | "send-channel-message"
        ) && self.positional[1].trim().is_empty()
        {
            return Err("message cannot be empty".into());
        }
        if matches!(self.command.as_str(), "create-channel" | "create-group")
            && self.positional[0].trim().is_empty()
        {
            return Err(if self.command == "create-group" {
                "group name cannot be empty"
            } else {
                "channel name cannot be empty"
            }
            .into());
        }
        if matches!(
            self.command.as_str(),
            "invite-to-channel"
                | "add-to-group"
                | "invite-to-group"
                | "remove-from-group"
                | "remove-from-channel"
        ) && self.positional[1]
            .parse::<u32>()
            .ok()
            .filter(|id| *id > 9)
            .is_none()
        {
            return Err("CONTACT_ID must be a regular contact ID greater than 9".into());
        }
        if self.option("--as-text").is_some() && self.option("--caption").is_some() {
            return Err(
                "--caption cannot be combined with --as-text; include the text in the file".into(),
            );
        }
        if matches!(
            self.command.as_str(),
            "send-file" | "send-voice" | "send-photo" | "send-webxdc"
        ) && !std::path::Path::new(&self.positional[1]).is_file()
        {
            return Err("file must be an existing local regular file".into());
        }
        Ok(())
    }
    pub(crate) fn option(&self, key: &str) -> Option<&str> {
        self.options.get(key).map(String::as_str)
    }
}

fn positive_id(value: &str) -> Result<u32> {
    value
        .parse::<u32>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| "ID must be a positive integer".into())
}
