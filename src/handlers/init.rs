use crate::Result;
use crate::cli::Args;
use crate::help::CommandHelp;
use deltachat::config::Config;
use deltachat::context::Context;

pub(crate) const HELP: CommandHelp = CommandHelp {
    name: "init",
    arguments: "",
    description: "Create a persistent bot identity and provision a chatmail account. Requires network access. Repeating init retains the identity and configured transport.",
    options: "  --name NAME                   Set display name (default: Sigma)\n  --profile-photo PATH          Set profile photo from a local image\n  --profile-photo-address PATH  Set profile photo (default: right-facing Sigma logo)\n  --account-qr QR               Use a DCACCOUNT:/DCLOGIN: provisioning QR\n",
    example: "init --name \"My bot\" --data-path ./bot-data",
    summary: "Create a bot identity and provision a chatmail account",
};

pub(crate) async fn configure_profile(
    context: &Context,
    args: &Args,
    path: &std::path::Path,
) -> Result<()> {
    let new_account = !context.is_configured().await?;
    if new_account && context.get_config(Config::Displayname).await?.is_none() {
        context
            .set_config(Config::Displayname, Some("Sigma"))
            .await?;
    }
    if let Some(name) = args.option("--name") {
        context.set_config(Config::Displayname, Some(name)).await?;
    }
    if let Some(photo) = args
        .option("--profile-photo")
        .or_else(|| args.option("--profile-photo-address"))
    {
        let photo = std::fs::canonicalize(photo)?;
        context
            .set_config(
                Config::Selfavatar,
                Some(photo.to_str().ok_or("photo path must be UTF-8")?),
            )
            .await?;
    } else if new_account && context.get_config(Config::Selfavatar).await?.is_none() {
        // Embed the public branding asset so installed binaries need no checkout.
        let photo = path.join("sigma-default-avatar.png");
        std::fs::write(
            &photo,
            include_bytes!("../../landing/static/assets/logo/moai-right-far.png"),
        )?;
        let photo = std::fs::canonicalize(photo)?;
        context
            .set_config(
                Config::Selfavatar,
                Some(photo.to_str().ok_or("photo path must be UTF-8")?),
            )
            .await?;
    }
    Ok(())
}

pub(crate) async fn handle(context: &Context, args: &Args, path: &std::path::Path) -> Result<()> {
    context.set_config(Config::Bot, Some("1")).await?;
    configure_profile(context, args, path).await?;
    let provisioned = context.init_transports(args.option("--account-qr")).await;
    context.stop_io().await;
    provisioned?;
    let addresses = context
        .list_transports()
        .await?
        .into_iter()
        .map(|transport| transport.addr)
        .collect::<Vec<_>>();
    let human = format!(
        "Account ready in {}\n{}",
        path.display(),
        addresses.join("\n")
    );
    crate::output::record(
        "account_ready",
        vec![
            ("data_path", path.to_string_lossy().into_owned().into()),
            (
                "addresses",
                crate::output::Value::Array(
                    addresses
                        .into_iter()
                        .map(crate::output::Value::from)
                        .collect(),
                ),
            ),
        ],
        &human,
    );

    Ok(())
}
