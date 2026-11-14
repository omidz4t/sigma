#!/usr/bin/env python3
"""Example message reply; register with an explicit --match filter if desired."""
import json
import sys

event = json.load(sys.stdin)
print(json.dumps({'reply': 'Message hook received: ' + event['text']}))
