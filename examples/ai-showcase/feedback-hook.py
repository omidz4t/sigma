#!/usr/bin/env python3
import json,sys
from pathlib import Path
root=Path(__file__).resolve().parent
event=json.load(sys.stdin)
expected=json.loads((root/'result.json').read_text())['chat_id']
if event.get('chat_id')!=expected or event.get('command') not in ['/ok','/no']:raise SystemExit(0)
queue=root/'.feedback';queue.mkdir(exist_ok=True)
item={'command':event['command'],'args':event.get('args','')[:1000],'incoming_message_id':event['incoming_message_id']}
p=queue/f"{event['incoming_message_id']}.json"
if p.exists():raise SystemExit(0)
p.write_text(json.dumps(item))
print(json.dumps({'reply':"I'll revise the app and send a new version." if event['command']=='/no' else "Approved — I'll confirm the finished version."}))
