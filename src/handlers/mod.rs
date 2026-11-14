pub(crate) mod account_create;
pub(crate) mod account_default;
pub(crate) mod account_delete;
pub(crate) mod account_list;
pub(crate) mod add_contact;
pub(crate) mod add_to_group;
pub(crate) mod agent_act;
pub(crate) mod agent_calls;
pub(crate) mod agent_connect;
pub(crate) mod agent_list;
pub(crate) mod agent_master;
pub(crate) mod agent_model;
pub(crate) mod agent_models;
pub(crate) mod agent_permissions;
pub(crate) mod agent_stop;
pub(crate) mod app_bind;
pub(crate) mod app_events;
pub(crate) mod app_hook_add;
pub(crate) mod app_hook_list;
pub(crate) mod app_hook_remove;
pub(crate) mod app_info;
pub(crate) mod app_list;
pub(crate) mod app_run;
pub(crate) mod app_send;
pub(crate) mod app_start;
pub(crate) mod app_stop;
pub(crate) mod app_update;
pub(crate) mod channel_link;
pub(crate) mod chat_hook_add;
pub(crate) mod chat_hook_list;
pub(crate) mod chat_hook_remove;
pub(crate) mod chats;
pub(crate) mod contacts;
pub(crate) mod create_channel;
pub(crate) mod create_contact;
pub(crate) mod create_group;
pub(crate) mod give_qr;
pub(crate) mod help;
pub(crate) mod init;
pub(crate) mod invite_to_channel;
pub(crate) mod join_link;
pub(crate) mod listen;
pub(crate) mod listen_webxdc_realtime;
pub(crate) mod listen_webxdc_updates;
pub(crate) mod messages;
pub(crate) mod react;
pub(crate) mod reactions;
pub(crate) mod read_messages;
pub(crate) mod receive;
pub(crate) mod reply;
pub(crate) mod securejoin_id;
pub(crate) mod send_channel_message;
pub(crate) mod send_file;
pub(crate) mod send_message;
pub(crate) mod send_photo;
pub(crate) mod send_voice;
pub(crate) mod send_webxdc;
pub(crate) mod send_webxdc_realtime;
pub(crate) mod send_webxdc_update;
pub(crate) mod status;
pub(crate) mod version;
pub(crate) mod webxdc_updates;

pub(crate) mod backup;
pub(crate) mod channel_members;
pub(crate) mod delete_message;
pub(crate) mod download_file;
pub(crate) mod edit_message;
pub(crate) mod export_file;
pub(crate) mod forward_message;
pub(crate) mod group_link;
pub(crate) mod group_members;
pub(crate) mod invite_to_group;
pub(crate) mod join_channel;
pub(crate) mod join_group;
pub(crate) mod leave_channel;
pub(crate) mod leave_group;
pub(crate) mod remove_from_channel;
pub(crate) mod remove_from_group;
pub(crate) mod rename_channel;
pub(crate) mod rename_group;
pub(crate) mod restore;
pub(crate) mod search_messages;
pub(crate) mod set_channel_description;
pub(crate) mod set_channel_photo;
pub(crate) mod set_group_photo;
pub(crate) mod set_profile_photo;

use crate::{Result, account::Account, cli::Args};

pub(crate) async fn dispatch(account: &Account, args: &Args) -> Result<()> {
    match args.command.as_str() {
        "agent-connect" => agent_connect::handle(account, args).await,
        "chat-hook-add" => chat_hook_add::handle(account, args).await,
        "chat-hook-list" => chat_hook_list::handle(account, args).await,
        "chat-hook-remove" => chat_hook_remove::handle(account, args).await,
        "app-bind" => app_bind::handle(account, args).await,
        "app-run" => app_run::handle(account, args).await,
        "add-contact" => add_contact::handle(&account.context, args).await,
        "add-to-group" => add_to_group::handle(&account.context, args).await,
        "channel-link" => channel_link::handle(&account.context, args).await,
        "chats" => chats::handle(&account.context, args).await,
        "contacts" => contacts::handle(&account.context, args).await,
        "create-channel" => create_channel::handle(&account.context, args).await,
        "create-contact" => create_contact::handle(&account.context, args).await,
        "create-group" => create_group::handle(&account.context, args).await,
        "give-qr" => give_qr::handle(&account.context, args, &account.path).await,
        "init" => init::handle(&account.context, args, &account.path).await,
        "invite-to-channel" => invite_to_channel::handle(&account.context, args).await,
        "join-link" => join_link::handle(&account.context, args).await,
        "listen" => listen::handle(&account.context, args, &account.path).await,
        "messages" => messages::handle(&account.context, args).await,
        "react" => react::handle(&account.context, args).await,
        "reactions" => reactions::handle(&account.context, args).await,
        "read-messages" => read_messages::handle(&account.context, args).await,
        "receive" => receive::handle(&account.context, args).await,
        "reply" => reply::handle(&account.context, args).await,
        "securejoin-id" => securejoin_id::handle(&account.context, args).await,
        "send-channel-message" => send_channel_message::handle(&account.context, args).await,
        "send-file" => send_file::handle(&account.context, args).await,
        "send-message" => send_message::handle(&account.context, args).await,
        "send-photo" => send_photo::handle(&account.context, args).await,
        "send-voice" => send_voice::handle(&account.context, args).await,
        "send-webxdc" => send_webxdc::handle(&account.context, args, &account.path).await,
        "send-webxdc-update" => send_webxdc_update::handle(&account.context, args).await,
        "webxdc-updates" => webxdc_updates::handle(&account.context, args).await,
        "listen-webxdc-updates" => listen_webxdc_updates::handle(&account.context, args).await,
        "send-webxdc-realtime" => send_webxdc_realtime::handle(&account.context, args).await,
        "listen-webxdc-realtime" => listen_webxdc_realtime::handle(&account.context, args).await,
        "status" => status::handle(&account.context, args).await,
        "group-members" => group_members::handle(&account.context, args).await,
        "remove-from-group" => remove_from_group::handle(&account.context, args).await,
        "rename-group" => rename_group::handle(&account.context, args).await,
        "channel-members" => channel_members::handle(&account.context, args).await,
        "remove-from-channel" => remove_from_channel::handle(&account.context, args).await,
        "rename-channel" => rename_channel::handle(&account.context, args).await,
        "set-channel-description" => set_channel_description::handle(&account.context, args).await,
        "set-profile-photo" => set_profile_photo::handle(&account.context, args).await,
        "set-group-photo" => set_group_photo::handle(&account.context, args).await,
        "set-channel-photo" => set_channel_photo::handle(&account.context, args).await,
        "leave-group" => leave_group::handle(&account.context, args).await,
        "leave-channel" => leave_channel::handle(&account.context, args).await,
        "group-link" => group_link::handle(&account.context, args).await,
        "invite-to-group" => invite_to_group::handle(&account.context, args).await,
        "join-group" => join_group::handle(&account.context, args).await,
        "join-channel" => join_channel::handle(&account.context, args).await,
        "search-messages" => search_messages::handle(&account.context, args).await,
        "delete-message" => delete_message::handle(&account.context, args).await,
        "edit-message" => edit_message::handle(&account.context, args).await,
        "forward-message" => forward_message::handle(&account.context, args).await,
        "export-file" => export_file::handle(&account.context, args).await,
        "download-file" => download_file::handle(&account.context, args).await,
        "backup" => backup::handle(&account.context, args).await,
        "restore" => restore::handle(&account.context, args).await,
        _ => Err(format!("unknown command: {} (see --help)", args.command).into()),
    }
}

pub(crate) async fn dispatch_client(args: &Args) -> Result<bool> {
    match args.command.as_str() {
        "account-create" => account_create::handle(args).await?,
        "account-list" => account_list::handle(args).await?,
        "account-default" => account_default::handle(args).await?,
        "account-delete" => account_delete::handle(args).await?,
        "agent-list" => agent_list::handle(args).await?,
        "agent-model" => agent_model::handle(args).await?,
        "agent-models" => agent_models::handle(args).await?,
        "agent-master" => agent_master::handle(args).await?,
        "agent-calls" | "agent-call-cancel" => agent_calls::handle(args).await?,
        "agent-permissions" => agent_permissions::handle(args).await?,
        "agent-act" => agent_act::handle(args).await?,
        "agent-stop" => agent_stop::handle(args).await?,
        "agent-connect" if args.option("--foreground").is_none() => {
            crate::agents::start(args).await?
        }
        "app-hook-add" => app_hook_add::handle(args).await?,
        "app-hook-list" => app_hook_list::handle(args).await?,
        "app-hook-remove" => app_hook_remove::handle(args).await?,

        "app-list" => app_list::handle(args).await?,
        "app-info" => app_info::handle(args).await?,
        "app-start" => app_start::handle(args).await?,
        "app-stop" => app_stop::handle(args).await?,
        "app-send" => app_send::handle(args).await?,
        "app-update" => app_update::handle(args).await?,
        "app-events" => app_events::handle(args).await?,
        _ => return Ok(false),
    }
    Ok(true)
}
