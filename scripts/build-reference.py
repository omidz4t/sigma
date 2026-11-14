#!/usr/bin/env python3
"""Build the complete Markdown reference and standalone browser reference."""
from pathlib import Path
import html
import re
from urllib.parse import quote

ROOT = Path(__file__).resolve().parent.parent
SOURCES = ['README.md', 'docs/cli.md', 'docs/state.md', 'docs/AI.md',
           'docs/chat-hooks.md', 'docs/app-hooks.md', 'docs/automation.md', 'docs/agent-actions.md', 'docs/agent-permissions.md', 'docs/agent-models.md', 'docs/agent-calls.md', 'docs/agent-security.md',
           'docs/webxdc.md', 'docs/webxdc-realtime.md', 'docs/voice.md',
           'docs/output.md', 'docs/backups.md', 'docs/releases.md',
           'docs/development.md', 'docs/testing.md']
sections = []
for index, source in enumerate(SOURCES):
    text = (ROOT / source).read_text()
    title = text.splitlines()[0].removeprefix('# ')
    sections.append((f'section-{index + 1}', title, source, text))

# Resolve links between included documents to the corresponding combined section.
def links(text, source):
    def replace(match):
        label, target = match.groups()
        if target.startswith(('https://', 'http://', '#')):
            return match.group()
        path = (ROOT / source).parent / target.split('#')[0]
        if path.resolve() in [(ROOT / 'docs/REFERENCE.md').resolve(), (ROOT / 'landing/static/reference.html').resolve()]:
            return f'[{label}](#reference-top)'
        for anchor, _, name, _ in sections:
            if path.resolve() == (ROOT / name).resolve():
                return f'[{label}](#{anchor})'
        if path.exists():
            return f'[{label}](../{path.resolve().relative_to(ROOT).as_posix()})'
        return match.group()
    return re.sub(r'\[([^\]]+)\]\(([^)]+)\)', replace, text)

intro = '''# Sigma — complete reference

Everything about the project in one document: getting started, commands, chat and app hooks, webxdc, state, output, backups, distribution, development, and verification.

Generated from the project documentation with `python3 scripts/build-reference.py`. Update the source guides, then regenerate this reference. Command-specific `--help` is the authority for supported arguments.

## Contents

'''
markdown = '<a id="reference-top"></a>\n\n' + intro + '\n'.join(f'- [{title}](#{anchor})' for anchor, title, _, _ in sections) + '\n\n'
for anchor, title, source, text in sections:
    markdown += f'<a id="{anchor}"></a>\n\n' + links(text, source) + '\n\n---\n\n'
(ROOT / 'docs/REFERENCE.md').write_text(markdown)

def inline(text):
    # Protect inline code and links before handling emphasis.
    tokens = []
    def token(value):
        tokens.append(value)
        return f'\x00{len(tokens)-1}\x00'
    text = html.escape(text)
    text = re.sub(r'`([^`]+)`', lambda m: token('<code>'+m[1]+'</code>'), text)
    text = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', lambda m: token('<a href="'+m[2]+'">'+m[1]+'</a>'), text)
    text = re.sub(r'\*\*([^*]+)\*\*', r'<strong>\1</strong>', text)
    return re.sub(r'\x00(\d+)\x00', lambda m: tokens[int(m[1])], text)

def render(text):
    out = []; paragraph = []; code = None; items = False
    def flush():
        if paragraph:
            out.append('<p>'+inline(' '.join(paragraph))+'</p>');paragraph.clear()
    for line in text.splitlines():
        if line.startswith('```'):
            flush()
            if items:out.append('</ul>');items=False
            if code is None:code=[]
            else:out.append('<pre><code>'+html.escape('\n'.join(code))+'</code></pre>');code=None
            continue
        if code is not None:code.append(line);continue
        if line.startswith('- '):
            flush()
            if not items:out.append('<ul>');items=True
            out.append('<li>'+inline(line[2:])+'</li>');continue
        if items:out.append('</ul>');items=False
        heading=re.match(r'^(#{1,6}) (.*)',line)
        if heading:
            flush();level=min(len(heading[1])+1,6);out.append(f'<h{level}>'+inline(heading[2])+f'</h{level}>')
        elif not line.strip():flush()
        elif line.startswith('|'):
            flush()
            if re.match(r'^\|[ :|\-]+$',line):continue
            cells=[inline(x.strip()) for x in line.strip('|').split('|')]
            out.append('<div class="table-row">'+''.join('<span>'+c+'</span>' for c in cells)+'</div>')
        else:paragraph.append(line)
    flush()
    if items:out.append('</ul>')
    return '\n'.join(out)

nav=''.join(f'<a href="#{anchor}">{html.escape(title)}</a>' for anchor,title,_,_ in sections)
body=''.join(f'<section id="{anchor}">{render(links(text,source))}</section>' for anchor,_,source,text in sections)
# Repository file links must also work when landing/ is deployed as the site root.
body = body.replace('href="../', 'href="https://github.com/omidz4t/sigma/blob/main/')
favicon = 'data:image/svg+xml,' + quote((ROOT / 'landing/static/logo.svg').read_text(), safe='')
page='''<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Sigma — complete reference</title><link rel="icon" type="image/svg+xml" href="'''+favicon+'''"><style>
:root{color-scheme:light}*{box-sizing:border-box}html{scroll-behavior:smooth;scroll-padding-top:25px}body{margin:0;background:#f5f4ed;color:#232b26;font:15px/1.8 Arial,sans-serif}a{color:#345f24;text-underline-offset:3px}aside{position:fixed;inset:0 auto 0 0;width:270px;overflow:auto;padding:35px 25px;border-right:1px solid #d9ddd2}aside>a{display:block;font-size:13px;padding:5px 0}aside .brand{font-size:25px;font-weight:bold;margin-bottom:22px;text-decoration:none}main{max-width:1100px;margin-left:270px;padding:45px 65px}h1{font-size:42px;letter-spacing:-1.5px;line-height:1.15}h2{font-size:30px;line-height:1.3}h3{font-size:23px}h4{font-size:18px}section{border-top:1px solid #d9ddd2;margin-top:50px;padding-top:20px;overflow-wrap:anywhere}code{font-family:ui-monospace,Consolas,monospace;font-size:.87em}pre{background:#222b26;color:#e1e9db;border-radius:7px;padding:20px;overflow:auto;line-height:1.7}p code,li code{background:#e5e9dc;padding:2px 4px;border-radius:3px}.table-row{display:flex;border-bottom:1px solid #d9ddd2;gap:12px;font-size:12px;padding:8px}.table-row span{flex:1;min-width:0}a:focus-visible{outline:2px solid #52822b;outline-offset:3px}@media(max-width:850px){aside{position:static;width:auto;border-right:0;border-bottom:1px solid #d9ddd2;padding:20px}aside>a:not(.brand){display:inline-block;margin-right:16px}main{margin:0;padding:25px 20px}h1{font-size:34px}.table-row{flex-wrap:wrap}.table-row span{min-width:120px}}@media print{aside{display:none}main{margin:0;padding:0;max-width:none}pre{white-space:pre-wrap;background:#eee;color:#111}section{break-before:auto}a{color:inherit}}@media(prefers-reduced-motion:reduce){html{scroll-behavior:auto}}
@media(prefers-color-scheme:dark){:root{color-scheme:dark}body{background:#171c18;color:#e7ece3}a{color:#b6d892}aside,section,.table-row{border-color:#374236}pre{background:#101611;color:#e1e9db}p code,li code{background:#2c3928}a:focus-visible{outline-color:#b6d892}}@media print{body{background:#fff;color:#111}a{color:inherit}}
</style></head><body><aside><a class="brand" href="./" aria-label="Sigma reference"><span aria-hidden="true">🗿</span> sigma / reference</a>NAVIGATE'''+nav+'''</aside><main id="reference-top"><h1>One place.<br>The complete sigma reference.</h1><p>Setup, every command, chat hooks, webxdc, persistent state, and development. Use your browser’s search to find a command. This page is self-contained and also works offline.</p>'''+body+'''</main></body></html>'''
(ROOT / 'landing/static/reference.html').write_text(page)
print('Built docs/REFERENCE.md and landing/static/reference.html')
