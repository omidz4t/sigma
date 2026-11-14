#!/usr/bin/env python3
"""Snapshot the public GitHub star count for the static site. No token is saved."""
import datetime
import json
import os
from pathlib import Path
import urllib.request

repository = 'omidz4t/sigma'
headers = {'Accept': 'application/vnd.github+json', 'User-Agent': 'sigma-pages'}
if token := os.environ.get('GH_TOKEN'):
    headers['Authorization'] = f'Bearer {token}'
request = urllib.request.Request(f'https://api.github.com/repos/{repository}', headers=headers)
with urllib.request.urlopen(request, timeout=30) as response:
    data = json.load(response)
stars = data.get('stargazers_count')
if type(stars) is not int or stars < 0 or data.get('full_name', '').lower() != repository:
    raise SystemExit('GitHub returned invalid repository statistics')
output = Path(__file__).resolve().parents[1] / 'landing/src/lib/repo-stats.json'
output.write_text(json.dumps({
    'stars': stars,
    'updatedAt': datetime.datetime.now(datetime.UTC).isoformat(),
}, indent=2) + '\n')
print(f'Prepared public star count: {stars}')
