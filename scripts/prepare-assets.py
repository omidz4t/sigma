#!/usr/bin/env python3
"""Generate ignored example archives and public live-test fixtures from source."""
from pathlib import Path
import math
import struct
import wave
from zipfile import ZIP_STORED, ZipFile, ZipInfo

root = Path(__file__).resolve().parent.parent

def archive(path, files):
    with ZipFile(path, 'w', compression=ZIP_STORED) as output:
        for name, data in files:
            info = ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            output.writestr(info, data)

for source in sorted((root / 'examples').iterdir()):
    if source.is_dir() and (source / 'index.html').is_file():
        archive(source.with_suffix('.xdc'), [(p.relative_to(source).as_posix(), p.read_bytes()) for p in sorted(source.rglob('*')) if p.is_file()])
fixtures = root / 'tests/fixtures'
(fixtures / 'photo.png').write_bytes(bytes.fromhex((fixtures / 'photo.hex').read_text()))
archive(fixtures / 'minimal.xdc', [('index.html', b'<!doctype html><title>Test</title><p>Sigma test app</p>')])
with wave.open(str(fixtures / 'voice-tone.wav'), 'wb') as output:
    output.setparams((1, 2, 16000, 16000, 'NONE', 'not compressed'))
    output.writeframes(b''.join(struct.pack('<h', int(math.sin(i * 440 * math.tau / 16000) * 4000)) for i in range(16000)))
print('Generated public example apps and synthetic fixtures (ignored by Git).')
