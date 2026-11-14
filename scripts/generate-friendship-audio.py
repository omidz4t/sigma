#!/usr/bin/env python3
"""Generate Gemini narration through ~/ai_local and align its words locally.

Requires: pip install faster-whisper imageio-ffmpeg
Run from the repository root. Raw speech stays in ignored private storage.
"""
import argparse
import difflib
import json
import re
import subprocess
import unicodedata
import wave
import numpy as np
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CACHE = ROOT / 'private/friendship-audio'
OUT = ROOT / 'landing/static/assets/friendship/audio'
MODEL = 'openrouter/google/gemini-3.8-flash-tts'


def blocks_for(lang):
    script = "import {narrationBlocks} from './landing/src/lib/friendship-data.js'; console.log(JSON.stringify(narrationBlocks));"
    blocks = json.loads(subprocess.check_output(['node', '--input-type=module', '-e', script], cwd=ROOT))
    dictionary = json.loads((ROOT / f'landing/src/lib/locales/{lang}.json').read_text())
    return [{**b, 'text': dictionary.get(b['text'], b['text'])} for b in blocks]


def normalized(text):
    text = unicodedata.normalize('NFKC', text).lower().replace('ي', 'ی').replace('ك', 'ک')
    return ''.join(c for c in text if c.isalnum())


def produce(lang, recognizer, ffmpeg):
    blocks = blocks_for(lang)
    pcm = CACHE / f'{lang}.pcm'
    if not pcm.exists():
        result = subprocess.run([str(Path.home() / 'ai_local'), '--timeout', '240s', 'voice', '-m', MODEL,
            '--voice', 'Kore', '--format', 'pcm', '--text', '\n\n'.join(b['text'] for b in blocks),
            '--out', str(pcm), '--json'], capture_output=True, text=True, timeout=260)
        (CACHE / f'{lang}-generation.log').write_text(result.stdout + result.stderr)
        result.check_returncode()
    # ai_local's Gemini speech endpoint returns 24 kHz, mono, signed 16-bit PCM.
    duration = pcm.stat().st_size / 48000
    wav = CACHE / f'{lang}.wav'
    for target, encoding in [(wav, ['-ar', '16000']), (OUT / f'{lang}.mp3', ['-codec:a', 'libmp3lame', '-b:a', '40k'])]:
        subprocess.run([ffmpeg, '-y', '-v', 'error', '-f', 's16le', '-ar', '24000', '-ac', '1',
            '-i', str(pcm), *encoding, str(target)], check=True)
    transcript = CACHE / f'{lang}-transcript.json'
    if transcript.exists():
        recognized = json.loads(transcript.read_text())
    else:
        with wave.open(str(wav), 'rb') as stream:
            samples = np.frombuffer(stream.readframes(stream.getnframes()), dtype=np.int16).astype(np.float32) / 32768
        segments, _ = recognizer.transcribe(samples, language=lang, word_timestamps=True,
            beam_size=5, vad_filter=True, condition_on_previous_text=False)
        recognized = [{'text': w.word, 'start': w.start, 'end': w.end}
            for segment in segments for w in segment.words]
        transcript.write_text(json.dumps(recognized, ensure_ascii=False, indent=2))
    source = [{'block': b['id'], 'word': i, 'text': word}
        for b in blocks for i, word in enumerate(re.findall(r'\S+', b['text']))]
    a, b = [normalized(w['text']) for w in source], [normalized(w['text']) for w in recognized]
    anchors = {}
    for match in difflib.SequenceMatcher(None, a, b, autojunk=False).get_matching_blocks():
        for delta in range(match.size): anchors[match.a + delta] = recognized[match.b + delta]
    if len(anchors) < len(source) * .35 or max(anchors, default=0) < len(source) * .93:
        raise RuntimeError(f'{lang}: speech/transcript coverage needs review before publishing')
    # Anchor timing to recognized speech; interpolate only unrecognized source words.
    boundaries = [-1, *sorted(anchors), len(source)]
    times = {}
    for i, anchor in anchors.items(): times[i] = (anchor['start'], anchor['end'])
    for left, right in zip(boundaries, boundaries[1:]):
        start = anchors[left]['end'] if left >= 0 else 0
        end = anchors[right]['start'] if right < len(source) else duration
        weights = [max(1, len(a[i])) for i in range(left + 1, right)]
        total = sum(weights) or 1
        cursor = start
        for i, weight in zip(range(left + 1, right), weights):
            following = cursor + max(0, end - start) * weight / total
            times[i] = (cursor, following); cursor = following
    words = []
    previous = 0
    for i, token in enumerate(source):
        start, end = times[i]
        start = max(previous, start)
        words.append({'block': token['block'], 'word': token['word'], 'start': round(start, 3), 'end': round(max(start + .02, end), 3)})
        previous = start
    (OUT / f'{lang}.json').write_text(json.dumps({'language': lang, 'duration': duration,
        'model': MODEL, 'voice': 'Kore', 'alignment': f'Whisper {"small" if lang == "fa" else "base"} word timestamps with source-text interpolation for unmatched words',
        'words': words}, ensure_ascii=False, separators=(',', ':')) + '\n')
    print(f'{lang}: {duration:.1f}s, {len(source)} words, {len(anchors)/len(source):.0%} direct word anchors', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('languages', nargs='*', default=['en', 'de', 'fa', 'ru'])
    args = parser.parse_args()
    import imageio_ffmpeg
    from faster_whisper import WhisperModel
    CACHE.mkdir(parents=True, exist_ok=True); OUT.mkdir(parents=True, exist_ok=True)
    recognizer = WhisperModel('base', device='cpu', compute_type='int8', cpu_threads=6)
    for lang in args.languages:
        selected = WhisperModel('small', device='cpu', compute_type='int8', cpu_threads=6) if lang == 'fa' else recognizer
        produce(lang, selected, imageio_ffmpeg.get_ffmpeg_exe())
