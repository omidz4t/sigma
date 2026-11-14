use crate::Result;

pub(crate) struct CommandHelp {
    pub(crate) name: &'static str,
    pub(crate) arguments: &'static str,
    pub(crate) description: &'static str,
    pub(crate) options: &'static str,
    pub(crate) example: &'static str,
    pub(crate) summary: &'static str,
}

pub(crate) const COMMAND_HELP: &[CommandHelp] = &[
    crate::handlers::agent_act::HELP,
    crate::handlers::account_delete::HELP,
    crate::handlers::account_default::HELP,
    crate::handlers::account_list::HELP,
    crate::handlers::account_create::HELP,
    crate::handlers::agent_connect::HELP,
    crate::handlers::agent_list::HELP,
    crate::handlers::agent_model::HELP,
    crate::handlers::agent_models::HELP,
    crate::handlers::agent_master::HELP,
    crate::handlers::agent_calls::HELP,
    crate::handlers::agent_calls::CANCEL_HELP,
    crate::handlers::agent_permissions::HELP,
    crate::handlers::agent_stop::HELP,
    crate::handlers::chat_hook_add::HELP,
    crate::handlers::chat_hook_list::HELP,
    crate::handlers::chat_hook_remove::HELP,
    crate::handlers::app_hook_add::HELP,
    crate::handlers::app_hook_list::HELP,
    crate::handlers::app_hook_remove::HELP,
    crate::handlers::app_list::HELP,
    crate::handlers::app_info::HELP,
    crate::handlers::app_start::HELP,
    crate::handlers::app_stop::HELP,
    crate::handlers::app_send::HELP,
    crate::handlers::app_update::HELP,
    crate::handlers::app_events::HELP,
    crate::handlers::app_bind::HELP,
    crate::handlers::app_run::HELP,
    crate::handlers::group_members::HELP,
    crate::handlers::remove_from_group::HELP,
    crate::handlers::rename_group::HELP,
    crate::handlers::channel_members::HELP,
    crate::handlers::remove_from_channel::HELP,
    crate::handlers::rename_channel::HELP,
    crate::handlers::set_channel_description::HELP,
    crate::handlers::set_channel_photo::HELP,
    crate::handlers::set_group_photo::HELP,
    crate::handlers::set_profile_photo::HELP,
    crate::handlers::leave_group::HELP,
    crate::handlers::leave_channel::HELP,
    crate::handlers::group_link::HELP,
    crate::handlers::invite_to_group::HELP,
    crate::handlers::join_group::HELP,
    crate::handlers::join_channel::HELP,
    crate::handlers::search_messages::HELP,
    crate::handlers::delete_message::HELP,
    crate::handlers::edit_message::HELP,
    crate::handlers::forward_message::HELP,
    crate::handlers::export_file::HELP,
    crate::handlers::download_file::HELP,
    crate::handlers::backup::HELP,
    crate::handlers::restore::HELP,
    crate::handlers::init::HELP,
    crate::handlers::status::HELP,
    crate::handlers::add_contact::HELP,
    crate::handlers::create_contact::HELP,
    crate::handlers::securejoin_id::HELP,
    crate::handlers::join_link::HELP,
    crate::handlers::give_qr::HELP,
    crate::handlers::contacts::HELP,
    crate::handlers::send_message::HELP,
    crate::handlers::reply::HELP,
    crate::handlers::send_file::HELP,
    crate::handlers::send_photo::HELP,
    crate::handlers::send_voice::HELP,
    crate::handlers::send_webxdc::HELP,
    crate::handlers::send_webxdc_realtime::HELP,
    crate::handlers::listen_webxdc_realtime::HELP,
    crate::handlers::send_webxdc_update::HELP,
    crate::handlers::webxdc_updates::HELP,
    crate::handlers::listen_webxdc_updates::HELP,
    crate::handlers::create_channel::HELP,
    crate::handlers::create_group::HELP,
    crate::handlers::add_to_group::HELP,
    crate::handlers::channel_link::HELP,
    crate::handlers::invite_to_channel::HELP,
    crate::handlers::send_channel_message::HELP,
    crate::handlers::chats::HELP,
    crate::handlers::messages::HELP,
    crate::handlers::receive::HELP,
    crate::handlers::read_messages::HELP,
    crate::handlers::react::HELP,
    crate::handlers::reactions::HELP,
    crate::handlers::listen::HELP,
    crate::handlers::help::HELP,
];

pub(crate) fn general_help() -> String {
    let mut help = crate::handlers::help::GENERAL_HEADER.to_owned();
    for entry in COMMAND_HELP {
        let usage = format!("{} {}", entry.name, entry.arguments);
        help.push_str(&format!("  {:<28} {}\n", usage.trim_end(), entry.summary));
    }
    help.push_str(crate::handlers::help::GENERAL_OPTIONS);
    help
}

pub(crate) fn command_help(command: &str) -> Result<String> {
    let entry = COMMAND_HELP
        .iter()
        .find(|entry| entry.name == command)
        .ok_or_else(|| format!("unknown command: {command} (see --help)"))?;
    let account_options = if command.starts_with("account-") || command == "agent-act" {
        ""
    } else {
        "  --account NAME    Named account (default: saved selection, initially legacy)\n  --data-path PATH  Explicit account directory (overrides saved default)\n"
    };
    Ok(format!(
        "sigma {} — {}\n\nUSAGE: sigma {} {} [OPTIONS]\n\nOPTIONS:\n{}{}  --json            Emit JSON Lines on stdout\n  --help, -h        Show this command's help\n  --version, -V     Show version\n\nEXAMPLE:\n  ./sigma {}\n\nOptions may appear before or after arguments. Quote text, paths with spaces, and\ninvitation links. Use -- before literal arguments beginning with '-'.\nHelp never opens account storage or starts networking.",
        entry.name,
        entry.description,
        entry.name,
        entry.arguments,
        entry.options,
        account_options,
        entry.example
    ))
}
