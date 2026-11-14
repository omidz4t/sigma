#!/bin/sh
set -eu
prefix=${HOME:?HOME is required}/.local
binary=
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix) [ "$#" -ge 2 ] || { echo 'Missing --prefix value' >&2; exit 1; }; prefix=$2; shift 2 ;;
        --binary) [ "$#" -ge 2 ] || { echo 'Missing --binary value' >&2; exit 1; }; binary=$2; shift 2 ;;
        --help|-h) echo 'Usage: scripts/install.sh [--prefix PATH] [--binary PATH]'; echo 'Default prefix: ~/.local. Builds a locked release unless --binary is supplied.'; exit 0 ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done
[ -n "$prefix" ] || { echo "Prefix must not be empty" >&2; exit 1; }
if [ -z "$binary" ]; then
    cargo build --locked --release --manifest-path "$project_dir/Cargo.toml" --target-dir "$project_dir/target"
    binary=$project_dir/target/release/sigma
fi
[ -f "$binary" ] || { echo 'Binary does not exist' >&2; exit 1; }
install -d "$prefix/bin"
install -m 755 "$binary" "$prefix/bin/sigma"
echo "Installed $prefix/bin/sigma"
