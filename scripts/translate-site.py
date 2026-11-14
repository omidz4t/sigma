#!/usr/bin/env python3
"""Regenerate website dictionaries with ai_local and native-language writing prompts."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
LOCALES = ROOT / 'landing/src/lib/locales'
MODEL = 'openrouter/google/gemini-3.5-flash-lite'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('languages', nargs='*', choices=['de', 'fa', 'ru'], default=['de', 'fa', 'ru'])
    parser.add_argument('--ai_local', default=str(Path.home() / 'ai_local'))
    parser.add_argument('--missing-only', action='store_true', help='Translate only new strings, preserving existing translations')
    parser.add_argument('--check', action='store_true', help='Validate existing dictionaries without model calls')
    args = parser.parse_args()
    source = json.loads((LOCALES / 'en.json').read_text())
    for language in args.languages:
        destination = LOCALES / f'{language}.json'
        if args.check:
            data = json.loads(destination.read_text())
        else:
            existing = json.loads(destination.read_text()) if args.missing_only else {}
            pending = {key: value for key, value in source.items() if key not in existing}
            with tempfile.NamedTemporaryFile(mode='w', suffix='.json') as attachment:
                json.dump(pending, attachment, ensure_ascii=False)
                attachment.flush()
                result = subprocess.run([
                    args.ai_local, '--timeout', '240s', 'ask', '-m', MODEL,
                    '--system-prompt', str(ROOT / f'scripts/translation-prompts/{language}.md'),
                    '--attach', attachment.name, '--max-tokens', '28000', '--no-usage',
                    '-q', 'Translate every dictionary value. Preserve every key and {brand} placeholder exactly. Return the complete JSON dictionary only.'
                ], capture_output=True, text=True, timeout=280, check=True)
            data = existing | json.loads(re.sub(r'^```(?:json)?\s*|\s*```$', '', result.stdout.strip()))
        missing, extra = set(source) - set(data), set(data) - set(source)
        if missing or extra:
            raise ValueError(f'{language}: missing keys {missing}; unexpected keys {extra}')
        for key, value in data.items():
            if not isinstance(value, str) or not value.strip():
                raise ValueError(f'{language}: empty translation for {key}')
            if any(value.count(token) != key.count(token) for token in re.findall(r'\{[a-zA-Z_]+\}', key)):
                raise ValueError(f'{language}: missing or duplicated placeholder')
        if not args.check:
            destination.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
        print(f'{language}: {len(data)} translations validated')


if __name__ == '__main__':
    main()
