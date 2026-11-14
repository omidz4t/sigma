# Build, install and release packaging

Build with stable Rust (minimum 1.89), C/C++ tools, CMake, Perl, make and pkg-config:

```sh
cargo build --locked --release
scripts/install.sh --prefix "$HOME/.local"
```

The installer builds a locked release by default and installs `PREFIX/bin/sigma` (default `~/.local/bin/sigma`). `--binary PATH` installs an already built/downloaded executable without invoking Cargo. Add the installation directory to PATH. Installation replaces a binary at that destination but never changes the account directory.

```sh
scripts/install.sh --prefix ./test-install --binary ./target/release/sigma
scripts/package.sh --version 0.1.0
```

`package.sh` packages the existing release binary, README, license and docs into `dist/sigma-VERSION-OS-ARCH.tar.gz`, with a `.sha256` checksum file. `--binary PATH` and `--output DIRECTORY` override source/destination. It does not compile, push tags or publish anything. Versions allow letters, digits, dots, underscores and hyphens. Verify downloaded archives with `sha256sum -c` (or `shasum -a 256 -c` on macOS) before extracting/installing.

The standard CI workflow runs format checks, Clippy, offline tests and a Linux release build. Pushing a `v*` tag starts `.github/workflows/release.yml`; it can also be manually dispatched with an existing version tag. It checks out that tag, validates and builds on Linux and macOS, packages archives/checksums, and publishes only after both builds pass. Build jobs have read-only repository permissions; only the publish job has contents write permission.

Releases live at <https://github.com/omidz4t/sigma/releases>. Each release includes raw executables named `sigma-linux-x86_64` or `sigma-darwin-ARCH`, matching `.sha256` files, and documentation bundles. Executables are release assets, never Git source files. The AI prompt uses the version from Cargo.toml to link its matching release.

For a manual stable release, review/commit the source, create a version tag matching Cargo.toml (for example `v0.2.0`), and push the tag. Automatic testing releases use the Python workflow below. An explicit manual dispatch is an alternative for an existing tag; do not dispatch twice for an already published version. Windows packaging is not supplied. The install/package scripts target Unix systems.

Sample `.xdc` archives and media are generated from text source. `cargo test` generates its own fixtures through `build.rs`; use `python3 scripts/prepare-assets.py` for runnable example apps or live-test fixtures. Packaging runs that generator automatically. After documentation changes, regenerate `docs/REFERENCE.md` and `landing/static/reference.html` with `python3 scripts/build-reference.py`; after a release build, regenerate the prompt with `python3 scripts/prepare-landing.py`.

## Automatic versions and testing releases

Following [madmail's release workflow](https://github.com/themadorg/madmail/blob/main/docs/TDD/release-automation.md), Sigma uses a dependency-free Python 3.11+ script and Conventional Commits. After main passes CI, `feat:` bumps minor; `fix:`, `perf:` and `revert:` bump patch. A `!` or `BREAKING CHANGE:` bumps minor by default; only explicit `--allow-major` permits a major bump. Documentation, maintenance and CI commits alone do not release.

```sh
python3 scripts/release.py                 # preview; no writes
python3 scripts/release.py --check         # verify numeric versions agree
python3 scripts/release.py --write         # commit and tag locally (clean main)
python3 scripts/release.py --write --push  # atomically push main and tag
```

The script reads the highest merged stable/unstable version tag, updates `.version`, Cargo manifests, `CHANGELOG.md` and the AI prompt release links, then creates `chore(release): X.Y.Z [skip ci]` and annotated `vX.Y.Z-unstable`. Numeric package versions remain X.Y.Z. An atomic, non-forced push refuses concurrent remote changes. Retrying a prepared release does not bump it again. Use one release operator per checkout.

CI calls the release workflow directly after pushing: GitHub's default Actions token does not trigger a second tag workflow. Linux and macOS builds must pass before GitHub publishes the testing prerelease. Stable `vX.Y.Z` tags remain manually published via the existing tag workflow; choose a stable version newer than the last stable release. No release signing or production credentials are added. The script is an independent Sigma implementation of the workflow, not copied madmail source.
