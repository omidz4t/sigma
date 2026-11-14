use crate::{Result, cli::Args};
use deltachat::context::{Context, ContextBuilder};
use std::path::PathBuf;

pub(crate) struct Account {
    pub(crate) context: Context,
    pub(crate) path: PathBuf,
    _lock: AccountLock,
}

pub(crate) fn data_path(
    override_path: Option<&str>,
    home: Option<&std::ffi::OsStr>,
) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return Ok(PathBuf::from(path));
    }
    let home = home
        .filter(|value| !value.is_empty())
        .ok_or("HOME is not set; provide --data-path explicitly")?;
    let home = PathBuf::from(home);
    if !home.is_absolute() {
        return Err("HOME must be an absolute path; provide --data-path explicitly".into());
    }
    Ok(home.join(".config").join("sigma"))
}

pub(crate) async fn open(args: &Args) -> Result<Account> {
    let home = std::env::var_os("HOME");
    let path = crate::accounts::resolve(args, home.as_deref())?;
    let database = path.join("account.db");
    if !matches!(args.command.as_str(), "init" | "restore") && !database.is_file() {
        return Err("account does not exist; run sigma init with this --data-path first".into());
    }
    if args.command == "restore"
        && (database.try_exists()? || (path.is_dir() && std::fs::read_dir(&path)?.next().is_some()))
    {
        return Err(
            "restore requires a new or empty account directory; choose --data-path explicitly"
                .into(),
        );
    }
    if matches!(args.command.as_str(), "init" | "restore") {
        if let Some(photo) = args
            .option("--profile-photo")
            .or_else(|| args.option("--profile-photo-address"))
            && !std::path::Path::new(photo).is_file()
        {
            return Err("profile photo must be an existing local file".into());
        }
        std::fs::create_dir_all(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    // This lock prevents concurrent CLI processes from operating on the same account.
    let lock = AccountLock::acquire(path.join("sigma.lock"))?;
    // Recheck under the lock in case another process created an account after
    // the empty-directory check but before this process acquired the lock.
    if args.command == "restore" && database.try_exists()? {
        return Err("restore refuses an existing account database".into());
    }
    let context = ContextBuilder::new(database).open().await?;
    Ok(Account {
        context,
        path,
        _lock: lock,
    })
}

pub(crate) struct AccountLock(std::fs::File);
impl AccountLock {
    pub(crate) fn acquire(path: PathBuf) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)?;
        file.try_lock()
            .map_err(|_| "account is in use by another sigma process")?;
        Ok(Self(file))
    }
}
impl Drop for AccountLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
