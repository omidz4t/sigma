use crate::Result;

/// Print the package version for --version / -V without opening an account.
pub(crate) async fn handle() -> Result<()> {
    crate::output::record(
        "version",
        vec![("version", env!("CARGO_PKG_VERSION").into())],
        concat!("sigma ", env!("CARGO_PKG_VERSION")),
    );
    Ok(())
}
