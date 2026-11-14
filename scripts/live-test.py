#!/usr/bin/env python3
"""Opt-in relay integration tests using two disposable bot accounts, never the default account."""
import argparse
import contextlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time
import uuid


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', action='store_true', help='Explicitly authorize network account creation and test traffic')
    parser.add_argument('--binary', default='target/debug/sigma')
    parser.add_argument('--report', default='/tmp/sigma-live-report.json')
    parser.add_argument('--work-dir', help='Reuse an existing private test directory; retained on success')
    args = parser.parse_args()
    if not args.run:
        parser.error('--run is required; these tests create two relay accounts and send test messages')
    binary = str(Path(args.binary).resolve())
    repo = Path(__file__).resolve().parent.parent
    subprocess.run(['python3', str(repo / 'scripts/prepare-assets.py')], check=True)
    work = Path(args.work_dir).resolve() if args.work_dir else Path(tempfile.mkdtemp(prefix='sigma-live-'))
    work.mkdir(mode=0o700, parents=True, exist_ok=True)
    a, b = work / 'a', work / 'b'
    checks = []
    processes = []
    sequence = 0
    token = uuid.uuid4().hex[:12]

    def call(account, command, *values):
        result = subprocess.run([binary, command, *map(str, values), '--data-path', str(account), '--json'], capture_output=True, text=True, timeout=180)
        if result.returncode:
            raise RuntimeError(f'{command} failed: {result.stderr[-2000:]}')
        return [json.loads(line) for line in result.stdout.splitlines() if line.strip()]

    def check(name):
        checks.append({'name': name, 'status': 'passed'})
        print(f'PASS {name}', flush=True)

    @contextlib.contextmanager
    def listen(account, *options):
        nonlocal sequence
        sequence += 1
        log = work / f'listener-{sequence}.jsonl'
        err = work / f'listener-{sequence}.stderr'
        with log.open('w') as out, err.open('w') as error:
            process = subprocess.Popen([binary, 'listen', *map(str, options), '--data-path', str(account), '--json'], stdout=out, stderr=error)
            processes.append(process)
            time.sleep(0.3)
            if process.poll() is not None:
                raise RuntimeError(f'listen failed: {err.read_text()}')
            try:
                yield log
            finally:
                if process.poll() is None:
                    process.send_signal(signal.SIGINT)
                    try:
                        process.wait(timeout=15)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
                if process.returncode != 0:
                    raise RuntimeError(f'listen exited {process.returncode}: {err.read_text()[-1000:]}')

    def records(log):
        result = []
        for line in log.read_text().splitlines():
            try:
                result.append(json.loads(line))
            except json.JSONDecodeError:
                pass  # A writer may still be completing the last line.
        return result

    def wait_record(log, predicate, seconds=60):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            for record in records(log):
                if predicate(record):
                    return record
            time.sleep(0.25)
        raise RuntimeError('timed out waiting for incoming test traffic')

    def incoming(log, text):
        return wait_record(log, lambda row: row.get('type') == 'message' and row['message']['text'] == text)['message']

    def received(account, text, seconds=60):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            for row in call(account, 'receive', '--timeout', '15'):
                if row.get('type') == 'message' and row['message']['text'] == text:
                    return row['message']
            time.sleep(0.25)
        raise RuntimeError('receive did not find expected message')

    report = {'started': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), 'checks': checks}
    try:
        call(a, 'init', '--name', 'sigma integration A')
        call(b, 'init', '--name', 'sigma integration B')
        check('two isolated live account initializations')
        invitation = call(a, 'join-link')[0]['link']
        with listen(a):
            b_contact = call(b, 'create-contact', invitation, '--timeout', '90')[0]['contact_id']
        a_contact = next(row['contact_id'] for row in call(a, 'contacts') if row['key_contact'])
        check('secure join and key contact discovery')

        first = f'sigma integration text {token}'
        with listen(a, '--auto-reply', f'auto receipt {token}') as log:
            b_original = call(b, 'send-message', b_contact, first)[0]['message_id']
            a_original = incoming(log, first)
            received(b, f'auto receipt {token}')
        check('text sending, continuous receiving, JSON messages, fixed auto reply')
        with listen(b) as log:
            call(a, 'reply', a_original['message_id'], f'quoted reply {token}')
            reply = incoming(log, f'quoted reply {token}')
            assert reply['parent_id'] == b_original
            call(a, 'react', a_original['message_id'], '👍', '--timeout', '60')
            wait_record(log, lambda row: row.get('type') == 'event' and 'ReactionsChanged' in row.get('event', ''))
        assert '👍' in json.dumps(call(b, 'reactions', b_original), ensure_ascii=False)
        check('quoted replies and reaction synchronization')

        text_file = work / 'payload.txt'
        text_file.write_text(f'text file content {token}\nsecond line\n')
        exported = work / f'exported-{token}.txt'
        with listen(a) as log:
            call(b, 'send-file', b_contact, text_file, '--caption', f'file {token}')
            file_message = incoming(log, f'file {token}')
            call(b, 'send-file', b_contact, text_file, '--as-text')
            incoming(log, text_file.read_text().rstrip("\n"))
            call(b, 'send-photo', b_contact, repo / 'tests/fixtures/photo.png', '--caption', f'photo {token}')
            photo = incoming(log, f'photo {token}')
            assert photo['viewtype'] in ['Image', 'Gif']
            call(b, 'send-webxdc', b_contact, repo / 'tests/fixtures/minimal.xdc', '--caption', f'app {token}')
            app = incoming(log, f'app {token}')
            assert app['viewtype'] == 'Webxdc'
        call(a, 'export-file', file_message['message_id'], exported)
        assert exported.read_bytes() == text_file.read_bytes()
        call(a, 'download-file', file_message['message_id'], work / f'downloaded-{token}.txt')
        check('file attachment/text, photo, webxdc, attachment download/export')

        group = call(a, 'create-group', f'Live group {token}')[0]['chat_id']
        call(a, 'add-to-group', group, a_contact)
        assert any(row['contact_id'] == a_contact for row in call(a, 'group-members', group))
        call(a, 'rename-group', group, f'Renamed group {token}')
        call(a, 'invite-to-group', group, a_contact)
        group_link = call(a, 'group-link', group)[0]['link']
        with listen(a):
            b_group = call(b, 'join-group', group_link, '--timeout', '90')[0]['chat_id']
        with listen(b) as log:
            for command, payload, caption in [
                ('send-message', f'group text {token}', None),
                ('send-file', text_file, f'group file {token}'),
                ('send-photo', repo / 'tests/fixtures/photo.png', f'group photo {token}'),
                ('send-webxdc', repo / 'tests/fixtures/minimal.xdc', f'group app {token}'),
            ]:
                extra = ['--caption', caption] if caption else []
                call(a, command, group, payload, '--chat', *extra)
                message = incoming(log, caption or str(payload))
                assert message['chat_id'] == b_group
        check('group creation, membership, rename, invitation join, all send types with --chat')

        channel = call(a, 'create-channel', f'Channel {token}')[0]['chat_id']
        call(a, 'invite-to-channel', channel, a_contact)
        channel_link = call(a, 'channel-link', channel)[0]['link']
        with listen(a):
            b_channel = call(b, 'join-channel', channel_link, '--timeout', '90')[0]['chat_id']
        assert any(row['contact_id'] == a_contact for row in call(a, 'channel-members', channel))
        call(a, 'rename-channel', channel, f'Renamed channel {token}')
        call(a, 'set-channel-description', channel, 'Integration test updates')
        call(a, 'set-channel-photo', channel, repo / 'tests/fixtures/photo.png')
        with listen(b) as log:
            call(a, 'send-channel-message', channel, f'channel post {token}')
            assert incoming(log, f'channel post {token}')['chat_id'] == b_channel
        call(a, 'remove-from-channel', channel, a_contact)
        assert not any(row['contact_id'] == a_contact for row in call(a, 'channel-members', channel))
        call(b, 'leave-channel', b_channel)
        check('channel join, subscribers, editing, posting, removal and leave')

        call(b, 'edit-message', b_original, f'edited original {token}')
        with listen(a) as log:
            with listen(b):
                wait_record(log, lambda row: row.get('type') == 'event' and 'MsgsChanged' in row.get('event', ''))
                time.sleep(2)
        history = call(a, 'read-messages', a_original['chat_id'])
        assert any(row.get('message', {}).get('text') == f'edited original {token}' for row in history)
        assert call(a, 'search-messages', token, '--chat-id', a_original['chat_id'])
        call(b, 'forward-message', b_original, b_group)
        with listen(a) as log:
            with listen(b):
                forwarded = incoming(log, f'edited original {token}')
                assert forwarded['forwarded']
        call(b, 'delete-message', b_original)
        assert not any(row.get('message', {}).get('message_id') == b_original for row in call(b, 'read-messages', reply['chat_id']))
        check('message editing, search, forwarding, deletion and history')

        hook = work / 'hook.py'
        hook.write_text('#!/usr/bin/env python3\nimport json,sys\nm=json.load(sys.stdin)\nprint("hook reply: "+m["text"])\n')
        hook.chmod(0o700)
        with listen(a, '--on-message', hook, '--hook-timeout', '5') as log:
            text = f'hook input {token}'
            call(b, 'send-message', b_contact, text)
            incoming(log, text)
            received(b, 'hook reply: ' + text)
        check('executable JSON message hook and automatic quoted reply')

        secret = work / 'backup-password'; secret.write_text('integration password\n')
        backups = work / 'backups'
        exported_backup = call(a, 'backup', backups, '--passphrase-file', secret)[0]
        backup = next(Path(value) for value in exported_backup['files'] if value.endswith('.tar'))
        restored = work / f'restored-{token}'
        call(restored, 'restore', backup, '--passphrase-file', secret)
        assert call(restored, 'status')[0]['addresses'] == call(a, 'status')[0]['addresses']
        call(a, 'remove-from-group', group, a_contact)
        assert not any(row['contact_id'] == a_contact for row in call(a, 'group-members', group))
        call(b, 'leave-group', b_group)
        check('database-encrypted core backup restore and group removal/leave')
        report['status'] = 'passed'
    except Exception as error:
        report['status'] = 'failed'
        report['error'] = str(error)
        report['state_directory'] = str(work)
        print(f'FAIL {error}', flush=True)
    finally:
        for process in processes:
            if process.poll() is None:
                process.kill(); process.wait()
        report['finished'] = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())
        Path(args.report).write_text(json.dumps(report, indent=2) + '\n')
        if report['status'] == 'passed' and not args.work_dir:
            shutil.rmtree(work)
        print(f'Report: {args.report}', flush=True)
    return 0 if report['status'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
