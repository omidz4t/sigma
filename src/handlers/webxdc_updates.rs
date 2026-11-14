use crate::{Result, cli::Args, help::CommandHelp};
use deltachat::{
    context::Context,
    message::{Message, MsgId, Viewtype},
    webxdc::{StatusUpdateItem, StatusUpdateSerial},
};

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "webxdc-updates",
    arguments: "MESSAGE_ID",
    description: "Read stored status updates for one webxdc app instance. MESSAGE_ID is the local ID of the original app message, not a chat ID. Does not fetch mail. Each result contains an updates array and last_serial for resuming.",
    options: "  --after SERIAL     Only return updates newer than this local serial (default: 0)\n",
    example: "webxdc-updates 26 --after 0 --json",
    summary: "Read an app's stored status updates",
};

pub(crate) async fn instance(context: &Context, raw: &str) -> Result<MsgId> {
    let id = MsgId::new(raw.parse()?);
    if Message::load_from_db(context, id).await?.get_viewtype() != Viewtype::Webxdc {
        return Err("MESSAGE_ID must identify a webxdc app message".into());
    }
    Ok(id)
}

// Infer core's JSON value type through its public update structure: no extra dependency.
pub(crate) async fn updates(context: &Context, id: MsgId, after: u32) -> Result<(String, u32)> {
    let raw = context
        .get_webxdc_status_updates(id, StatusUpdateSerial::new(after))
        .await?;
    let parsed = StatusUpdateItem {
        payload: raw.parse()?,
        ..Default::default()
    };
    let items = parsed
        .payload
        .as_array()
        .ok_or("core returned a non-array update list")?;
    let last = items
        .iter()
        .filter_map(|item| item.get("serial")?.as_u64())
        .max()
        .unwrap_or(u64::from(after));
    Ok((parsed.payload.to_string(), last.try_into()?))
}

pub(crate) async fn emit(
    context: &Context,
    id: MsgId,
    after: u32,
    include_empty: bool,
) -> Result<u32> {
    let (json, last) = updates(context, id, after).await?;
    if include_empty || last > after {
        crate::output::webxdc_updates(id.to_u32(), &json, last);
    }
    Ok(last)
}

pub(crate) async fn handle(context: &Context, args: &Args) -> Result<()> {
    let id = instance(context, &args.positional[0]).await?;
    emit(
        context,
        id,
        args.option("--after").unwrap_or("0").parse()?,
        true,
    )
    .await?;
    Ok(())
}
