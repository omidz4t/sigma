#!/usr/bin/env python3
"""Handle explicit button events. Invoke via app-start NAME --on-event ./examples/app-hook.py."""
import json
from pathlib import Path
import sys

event = json.load(sys.stdin)
if event['type'] == 'realtime_received':
    try:
        payloads = [json.loads(event.get('text') or 'null')]
    except json.JSONDecodeError:
        payloads = []
elif event['type'] == 'webxdc_updates':
    payloads = [update.get('payload') for update in event['updates']]
else:
    payloads = []

buttons = [p for p in payloads if isinstance(p, dict) and
           (p.get('type') == 'button_clicked' or p.get('action') == 'button_clicked')]
if buttons:
    # This directory is supplied by sigma, not by the remote packet's payload.
    marker = Path(event['state_path']) / 'hook-calls.jsonl'
    with marker.open('a') as log:
        log.write(json.dumps({'event_seq': event['seq'], 'hook_id': event.get('hook_id', 'default'), 'buttons': buttons}) + '\n')
    # Ignore non-button updates, including our own response, to avoid reply loops.
    actions = {'update': {
        'payload': {'type': 'hook_response', 'event_seq': event['seq'], 'hook_id': event.get('hook_id', 'default'),
                    'text': 'Button received — the executable hook ran!'},
        'info': 'Button handled by bot hook',
        'notify': {'*': 'Your button click ran the bot hook'}
    }}
    if event['type'] == 'realtime_received':
        actions['send'] = json.dumps({'type': 'hook_response', 'event_seq': event['seq'], 'hook_id': event.get('hook_id', 'default'), 'text': 'Button received — the hook ran!'})
    print(json.dumps(actions))
