# Friendship story assets

Eight chapter illustrations were generated with the built-in imagegen tool, then resized and encoded as 480 px WebP assets for the website. The shared art direction was soft rounded cartoon shapes, sage/moss green, cream and lavender gray stone; a flat #92a47c backdrop, no vignette, text, or watermark. Each prompt described its chapter: an agent reaching a person, a private webpage preview, interactive feedback, realtime packets, a local Wi-Fi relay, encrypted delivery, a compact server, and the three tools holding hands.

Audio is generated through `~/ai_local voice`, using `openrouter/google/gemini-3.8-flash-tts`, voice Kore, in English, German, Persian and Russian. The spoken script is the translated article text in `friendship-data.js`. Regenerate with `scripts/generate-friendship-audio.py`. Local Whisper word timestamps anchor the read-along; unmatched words use interpolation. Audio and cues are bundled locally and need no runtime AI request.
