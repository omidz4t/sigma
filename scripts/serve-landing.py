#!/usr/bin/env python3
"""Launch the SvelteKit dev server at the legacy local preview address."""
import argparse
import os
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--port', type=int, default=8080)
args = parser.parse_args()
site = Path(__file__).resolve().parents[1] / 'landing'
if not (site / 'node_modules').exists():
    parser.exit(1, 'Install website dependencies first: npm ci --prefix landing\n')
os.chdir(site)
os.environ['BASE_PATH'] = '/landing'
os.execvp('npm', ['npm', 'run', 'dev', '--', '--port', str(args.port), '--strictPort'])
