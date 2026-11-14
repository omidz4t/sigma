#!/usr/bin/env python3
from pathlib import Path
import json,subprocess,sys,time,signal
root=Path(__file__).resolve().parent
mode=sys.argv[1];wanted='/no' if mode=='revise' else '/ok'
chat_id=json.loads((root/'result.json').read_text())['chat_id']
queue=root/'.feedback';queue.mkdir(exist_ok=True)
def run(args):
 print('\n$ '+' '.join(map(str,args)),flush=True)
 subprocess.run(list(map(str,args)),check=True,cwd=root)
if not (root/'.hooks-ready').exists():
 for name,command in [('revise','/no'),('approve','/ok')]:
  run(['sigma','chat-hook-add',chat_id,name,root/'feedback-hook.py','--command',command])
 (root/'.hooks-ready').write_text('registered')
print('\n$ sigma listen --json\nWaiting for '+wanted+' from the demo phone...',flush=True)
log=(root/'.private/listener.jsonl').open('a')
listener=subprocess.Popen(['sigma','listen','--json'],stdout=log,stderr=log,cwd=root)
try:
 deadline=time.monotonic()+600
 while True:
  items=[]
  for p in sorted(queue.glob('*.json')):
   if p.with_suffix('.done').exists():continue
   event=json.loads(p.read_text())
   if event['command']==wanted:items.append((p,event))
  if items:break
  if listener.poll() is not None:raise RuntimeError('Sigma listener exited before feedback')
  if time.monotonic()>deadline:raise TimeoutError('No demo feedback received')
  time.sleep(.3)
 p,event=items[0]
 print('\nReceived chat hook event: '+json.dumps(event),flush=True)
 p.with_suffix('.done').write_text('handled')
 # Hooks only enqueue work. Release the account before the agent sends again.
 time.sleep(2)
finally:
 listener.send_signal(signal.SIGINT)
 try:listener.wait(timeout=30)
 except subprocess.TimeoutExpired:listener.terminate();listener.wait(timeout=10)
 log.close()
if mode=='revise':
 prompt='Use the installed sigma skill. The user reviewed Orbit Desk on their phone and sent this /no feedback: '+event['args']+' Revise the existing app/index.html accordingly; preserve its working local interactivity and webxdc update support. Rebuild orbit-desk.xdc with the changed code and send the NEW app to the existing alias me using sigma send-webxdc --json. The account is already connected, so do not init or reconnect. Save the new returned IDs to result-revised.json. This is explicit permission to send that revised demo app. Do not print invite.txt or credentials. Stop after sending and summarize.'
else:
 prompt='Use the installed sigma skill. The user approved Orbit Desk from Delta Chat with /ok. Using the existing connected account and alias me, send exactly one text confirmation: Approved — Orbit Desk is ready. Save the returned IDs to result-approval.json. Do not create or send another app, do not init or reconnect, and do not print credentials or invite.txt. Stop after confirming.'
(root/f'prompt-{mode}.txt').write_text(prompt)
print('\n$ opencode run --continue --model opencode/glm-5.3-flash-free "Follow the '+wanted+' feedback"',flush=True)
subprocess.run(['opencode','run','--standalone','--auto','--continue','--model','opencode/glm-5.3-flash-free',prompt],cwd=root,check=True)
print('\nFeedback loop completed.',flush=True)
