#!/usr/bin/env python3
"""Register with --type command --command /ping."""
import json
import sys

event = json.load(sys.stdin)
print(json.dumps({'reply': 'pong' + (': ' + event['args'] if event['args'] else '')}))
