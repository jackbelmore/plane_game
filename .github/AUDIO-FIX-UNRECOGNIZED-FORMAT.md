# Audio panic: UnrecognizedFormat

If you see:

```
thread 'Compute Task Pool' panicked at ...\bevy_audio-0.15.3\src\audio_source.rs:102:56:
called `Result::unwrap()` on an `Err` value: UnrecognizedFormat
```

**Cause:** Bevy’s audio backend (rodio) doesn’t support the format of your WAV files. Common causes:

- WAV with non-PCM encoding (e.g. ADPCM, IEEE float in an unsupported layout)
- Wrong sample rate / bit depth
- Corrupt or non-standard WAV header

**Fix (Gemini / audio author):**

1. **Re-export as standard WAV**
   - 44.1 kHz or 48 kHz
   - 16-bit PCM
   - Mono or stereo
   - Use Audacity, ffmpeg, or similar:
     - **ffmpeg:** `ffmpeg -i engine.wav -ar 44100 -ac 1 -acodec pcm_s16le engine_fixed.wav`
     - Replace `engine.wav` / `missile.wav` / `explosion.wav` in `assets/sounds/` with the fixed files.

2. **Or use OGG**
   - Bevy supports OGG. Convert WAV → OGG and change `SoundAssets` to load `.ogg` paths instead of `.wav`.

3. **Don’t rely on “it plays in another app”**
   - Many players accept formats that rodio does not; re-export to plain PCM WAV or OGG.

After replacing the files, the panic should stop.
