#!/bin/sh
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
binary=$project_dir/target/release/sigma
output=$project_dir/dist
version=
while [ "$#" -gt 0 ]; do
    case "$1" in
        --binary|--output|--version)
            [ "$#" -ge 2 ] || { echo "Missing $1 value" >&2; exit 1; }
            case "$1" in --binary) binary=$2 ;; --output) output=$2 ;; --version) version=$2 ;; esac
            shift 2 ;;
        --help|-h) echo 'Usage: scripts/package.sh --version VERSION [--binary PATH] [--output DIRECTORY]'; exit 0 ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done
case "$version" in ''|*[!a-zA-Z0-9._-]*) echo 'A valid --version is required' >&2; exit 1 ;; esac
[ -f "$binary" ] || { echo 'Build the release binary first' >&2; exit 1; }
platform=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
name=sigma-$version-$platform-$arch
mkdir -p "$output"
output=$(CDPATH= cd -- "$output" && pwd)
staging=$(mktemp -d)
trap 'rm -rf "$staging"' EXIT HUP INT TERM
mkdir "$staging/$name"
install -m 755 "$binary" "$staging/$name/sigma"
cp "$project_dir/README.md" "$project_dir/LICENSE" "$project_dir/THIRD_PARTY_NOTICES.md" "$staging/$name/"
cp -R "$project_dir/LICENSES" "$staging/$name/LICENSES"
cp -R "$project_dir/docs" "$staging/$name/docs"
cp -R "$project_dir/examples" "$staging/$name/examples"
python3 "$project_dir/scripts/prepare-assets.py"
cp "$project_dir"/examples/*.xdc "$staging/$name/examples/"
tar -czf "$output/$name.tar.gz" -C "$staging" "$name"
(cd "$output" && if command -v sha256sum >/dev/null 2>&1; then sha256sum "$name.tar.gz"; else shasum -a 256 "$name.tar.gz"; fi) > "$output/$name.tar.gz.sha256"
echo "$output/$name.tar.gz"
