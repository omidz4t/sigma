//! Named accounts alongside the original ~/.config/sigma account.
use crate::{
    Result,
    account::{self, AccountLock},
    cli::Args,
    output,
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub(crate) fn registry(home: Option<&OsStr>) -> Result<PathBuf> {
    Ok(account::data_path(None, home)?.with_file_name("sigma-accounts"))
}
pub(crate) fn path(home: Option<&OsStr>, name: &str) -> Result<PathBuf> {
    crate::aliases::validate(name)?;
    if name == "legacy" {
        account::data_path(None, home)
    } else {
        Ok(registry(home)?.join("accounts").join(name))
    }
}
pub(crate) fn default_name(home: Option<&OsStr>) -> Result<String> {
    let file = registry(home)?.join("default-account");
    match std::fs::read_to_string(file) {
        Ok(value) => {
            let name = value.trim();
            crate::aliases::validate(name)?;
            Ok(name.into())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok("legacy".into()),
        Err(error) => Err(error.into()),
    }
}
pub(crate) fn resolve(args: &Args, home: Option<&OsStr>) -> Result<PathBuf> {
    if let Some(override_path) = args.option("--data-path") {
        return account::data_path(Some(override_path), home);
    }
    let name = args
        .option("--account")
        .map(str::to_owned)
        .map(Ok)
        .unwrap_or_else(|| default_name(home))?;
    path(home, &name)
}
fn private_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
fn lock(home: Option<&OsStr>) -> Result<AccountLock> {
    let root = registry(home)?;
    private_directory(&root)?;
    AccountLock::acquire(root.join("registry.lock"))
}

/// Reserve a new account directory before provisioning; existing identities are never reused.
pub(crate) fn reserve(home: Option<&OsStr>, name: &str) -> Result<PathBuf> {
    crate::aliases::validate(name)?;
    if name == "legacy" {
        return Err("legacy is reserved for the original account".into());
    }
    let _lock = lock(home)?;
    let target = path(home, name)?;
    private_directory(target.parent().ok_or("invalid account path")?)?;
    std::fs::create_dir(&target).map_err(|error| {
        format!(
            "cannot create account {name}: {error}; use --account {name} for an existing account"
        )
    })?;
    private_directory(&target)?;
    Ok(target)
}

pub(crate) async fn create(args: &Args, name: &str) -> Result<PathBuf> {
    let home = std::env::var_os("HOME");
    let target = reserve(home.as_deref(), name)?;
    let mut init = Args {
        command: "init".into(),
        positional: vec![],
        options: Default::default(),
    };
    init.options
        .insert("--data-path".into(), target.to_string_lossy().into_owned());
    for key in [
        "--name",
        "--profile-photo",
        "--profile-photo-address",
        "--account-qr",
    ] {
        if let Some(value) = args.option(key) {
            init.options.insert(key.into(), value.into());
        }
    }
    let account = account::open(&init).await?;
    crate::handlers::init::handle(&account.context, &init, &account.path).await?;
    Ok(target)
}

pub(crate) fn set_default(home: Option<&OsStr>, name: &str) -> Result<()> {
    let _lock = lock(home)?;
    let account = path(home, name)?;
    if !account.join("account.db").is_file() {
        return Err(format!("unknown account: {name}").into());
    }
    let root = registry(home)?;
    let temporary = root.join("default-account.tmp");
    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(&temporary)?;
    writeln!(file, "{name}")?;
    file.sync_all()?;
    std::fs::rename(temporary, root.join("default-account"))?;
    Ok(())
}

pub(crate) fn delete(home: Option<&OsStr>, name: &str) -> Result<()> {
    let _registry = lock(home)?;
    let target = path(home, name)?;
    if default_name(home)? == name {
        return Err(
            "cannot delete the default account; select another with account-default first".into(),
        );
    }
    if std::fs::symlink_metadata(&target)?.file_type().is_symlink() {
        return Err("refusing to delete a symlinked account directory".into());
    }
    if !target.is_dir() || (name == "legacy" && !target.join("account.db").is_file()) {
        return Err(format!("unknown account: {name}").into());
    }
    let _account = AccountLock::acquire(target.join("sigma.lock"))?;
    // Holding the registry and account locks prevents deletion of a running identity.
    std::fs::remove_dir_all(target)?;
    Ok(())
}

pub(crate) async fn manage(args: &Args) -> Result<()> {
    let home = std::env::var_os("HOME");
    match args.command.as_str() {
        "account-create" => {
            create(args, &args.positional[0]).await?;
        }
        "account-default" => {
            let name = &args.positional[0];
            set_default(home.as_deref(), name)?;
            output::record(
                "account_default",
                vec![("account", name.as_str().into())],
                &format!("Default account: {name}"),
            );
        }
        "account-delete" => {
            let name = &args.positional[0];
            delete(home.as_deref(), name)?;
            output::record(
                "account_deleted",
                vec![("account", name.as_str().into())],
                &format!("Deleted local account {name}"),
            );
        }
        "account-list" => {
            let default = default_name(home.as_deref())?;
            let mut names = Vec::new();
            if path(home.as_deref(), "legacy")?
                .join("account.db")
                .is_file()
            {
                names.push("legacy".to_owned());
            }
            let dir = registry(home.as_deref())?.join("accounts");
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir()
                        && let Some(name) = entry.file_name().to_str()
                        && crate::aliases::validate(name).is_ok()
                    {
                        names.push(name.to_owned());
                    }
                }
            }
            names.sort();
            if names.is_empty() {
                output::record(
                    "account_list",
                    vec![("count", 0_i64.into())],
                    "No accounts.",
                );
            }
            for name in names {
                let path = path(home.as_deref(), &name)?;
                let selected = name == default;
                output::record(
                    "account_entry",
                    vec![
                        ("account", name.as_str().into()),
                        ("default", selected.into()),
                        ("data_path", path.to_string_lossy().into_owned().into()),
                        ("database_exists", path.join("account.db").is_file().into()),
                    ],
                    &format!(
                        "{} {name}  {}",
                        if selected { "*" } else { " " },
                        path.display()
                    ),
                );
            }
        }
        _ => return Err("unknown account management command".into()),
    }
    Ok(())
}
