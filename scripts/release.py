#!/usr/bin/env python3
"""Version Sigma from Conventional Commits. Python 3.11+, no Python dependencies."""
import argparse
import datetime
import json
from pathlib import Path
import re
import subprocess
import tomllib

SEMVER = re.compile(r'v(\d+)\.(\d+)\.(\d+)(?:-unstable)?$')


def git(root, *args):
    result = subprocess.run(['git', *args], cwd=root, text=True, capture_output=True)
    if result.returncode:
        raise ValueError(f'git {args[0]} failed; no remote history was forced')
    return result.stdout.strip()


def severity(message):
    subject = message.splitlines()[0] if message else ''
    match = re.match(r'(\w+)(?:\([^\n)]+\))?(!)?: ', subject)
    if re.search(r'^BREAKING[ -]CHANGE:', message, re.M) or (match and match[2]):
        return 3
    return {'feat': 2, 'fix': 1, 'perf': 1, 'revert': 1}.get(match[1], 0) if match else 0


def plan(root, allow_major=False):
    version = (root / '.version').read_text().strip()
    manifest = tomllib.loads((root / 'Cargo.toml').read_text())
    lock = tomllib.loads((root / 'Cargo.lock').read_text())
    own = [p for p in lock['package'] if p['name'] == 'sigma' and 'source' not in p]
    if not re.fullmatch(r'\d+\.\d+\.\d+', version) or manifest['package']['version'] != version or len(own) != 1 or own[0]['version'] != version:
        raise ValueError('.version, Cargo.toml and Cargo.lock must agree')
    tags = [t for t in git(root, 'tag', '--merged', 'HEAD').splitlines() if SEMVER.fullmatch(t)]
    base = max(tags, key=lambda t: (tuple(map(int, SEMVER.fullmatch(t).groups())), t.endswith('-unstable')), default='')
    history = git(root, 'log', '--reverse', '--format=%B%x00', f'{base}..HEAD' if base else 'HEAD')
    messages = [m.strip() for m in history.split('\0') if m.strip()]
    changes = [m for m in messages if severity(m)]
    level = max(map(severity, changes), default=0)
    if not level:
        return version, base, {}, ''
    major, minor, patch = map(int, SEMVER.fullmatch(base).groups()) if base else map(int, version.split('.'))
    if level == 3 and allow_major:
        major, minor, patch = major + 1, 0, 0
    elif level >= 2:
        minor, patch = minor + 1, 0
    else:
        patch += 1
    new = f'{major}.{minor}.{patch}'
    if tuple(map(int, new.split('.'))) <= tuple(map(int, version.split('.'))):
        raise ValueError('Planned version must advance the current manifests')
    tag = f'v{new}-unstable'
    notes = f'## {tag} ({datetime.datetime.now(datetime.UTC).date()})\n\n'
    # Render commit subjects as code, escaping Markdown and HTML delimiters.
    notes += '\n'.join('- ' + m.splitlines()[0].replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;').replace('`', '\\`') for m in changes) + '\n'
    if level == 3:
        notes += '\nBreaking changes included; review commit bodies before upgrading.\n'
    updates = {'.version': new + '\n'}
    for filename, heading in [('Cargo.toml', '[package]'), ('Cargo.lock', '[[package]]')]:
        text = (root / filename).read_text()
        if filename == 'Cargo.toml':
            pattern = r'(?ms)(^\[package\]\n.*?^version = ")[^"]+(\")'
        else:
            pattern = r'(?m)(^name = "sigma"\nversion = ")[^"]+(\")'
        text, count = re.subn(pattern, lambda m: m[1] + new + m[2], text, count=1)
        if count != 1:
            raise ValueError(f'Cannot update {filename}')
        updates[filename] = text
    updates['CHANGELOG.md'] = '# Changelog\n\n' + notes + '\n' + (root / 'CHANGELOG.md').read_text().removeprefix('# Changelog\n').lstrip()
    # Keep the self-contained AI prompt pinned to the matching public artifacts.
    prompt = root / 'landing/src/lib/prompt-data.js'
    if prompt.exists():
        updates['landing/src/lib/prompt-data.js'] = re.sub(r'v' + re.escape(version) + r'(?:-unstable)?(?![0-9.])', tag, prompt.read_text()).replace(f'sigma {version}', f'sigma {new}')
    return new, tag, updates, notes


def release(root, write=False, push=False, check=False, allow_major=False):
    if push and not write:
        raise ValueError('--push requires --write')
    version, tag, updates, notes = plan(root, allow_major)
    head = git(root, 'rev-parse', 'HEAD')
    if check:
        return dict(version=(root / '.version').read_text().strip(), tag='', sha=head, released=False)
    if git(root, 'rev-parse', '--is-shallow-repository') != 'false':
        raise ValueError('Fetch full history and tags first')
    if write:
        if git(root, 'status', '--porcelain'):
            raise ValueError('Commit source changes first; release requires a clean worktree')
        if git(root, 'branch', '--show-current') != 'main':
            raise ValueError('Release writes require main')
        if updates:
            if git(root, 'tag', '--list', tag):
                raise ValueError('Release tag already exists')
            originals = {name: (root / name).read_text() for name in updates}
            try:
                for name, text in updates.items():
                    (root / name).write_text(text)
                git(root, 'add', '--', *updates)
                git(root, '-c', 'core.hooksPath=/dev/null', '-c', 'commit.gpgsign=false', 'commit', '-m', f'chore(release): {version} [skip ci]')
            except BaseException:
                for name, text in originals.items():
                    (root / name).write_text(text)
                git(root, 'reset', '--', *updates)
                raise
            head = git(root, 'rev-parse', 'HEAD')
        # A retry can finish tagging/pushing a previously prepared commit.
        expected = f'chore(release): {version} [skip ci]'
        if git(root, 'log', '-1', '--format=%s') == expected:
            tag = f'v{version}-unstable'
            if not git(root, 'tag', '--list', tag):
                git(root, '-c', 'tag.gpgsign=false', 'tag', '-a', tag, '-m', f'Sigma {version}')
            if git(root, 'rev-parse', f'{tag}^{{commit}}') != head:
                raise ValueError('Release tag points at another commit')
            if push:
                git(root, 'push', '--atomic', 'origin', f'{head}:refs/heads/main', f'refs/tags/{tag}:refs/tags/{tag}')
            return dict(version=version, tag=tag, sha=head, released=True)
    return dict(version=version, tag=tag if updates else '', sha=head, released=bool(updates), dry_run=not write, notes=notes, files=list(updates))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path(__file__).resolve().parents[1])
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--write', action='store_true', help='commit and tag on main')
    mode.add_argument('--dry-run', action='store_true', help='preview only (default)')
    mode.add_argument('--check', action='store_true', help='check version consistency')
    parser.add_argument('--push', action='store_true', help='atomically push main and tag')
    parser.add_argument('--allow-major', action='store_true')
    parser.add_argument('--github-output', type=Path)
    args = parser.parse_args()
    try:
        result = release(args.repo.resolve(), args.write, args.push, args.check, args.allow_major)
        print(json.dumps(result, indent=2))
        if args.github_output:
            with args.github_output.open('a') as output:
                for key in ('version', 'tag', 'sha', 'released'):
                    output.write(f'{key}={str(result[key]).lower() if isinstance(result[key], bool) else result[key]}\n')
    except (ValueError, OSError) as error:
        parser.exit(1, f'release: {error}\n')


if __name__ == '__main__':
    main()
