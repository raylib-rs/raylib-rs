# Audio

Audio in raylib-rs sits behind a separate device token:
[`RaylibAudio`](https://docs.rs/raylib/latest/raylib/core/audio/struct.RaylibAudio.html).
Call `RaylibAudio::init_audio_device()` to open the audio device and get the token.
Every audio resource — `Wave`, `Sound`, `Music`, `AudioStream` — is created through
methods on `RaylibAudio` and carries a **lifetime tied to the `RaylibAudio` token**.
The borrow checker enforces that no audio resource outlives the device, so teardown
order is correct by construction.

> **Note on the plan's wording.** The plan refers to "AudioHandle" — the real type
> name in the codebase is `RaylibAudio` (`raylib::core::audio::RaylibAudio`).

## API surface

- [`RaylibAudio::init_audio_device`](https://docs.rs/raylib/latest/raylib/core/audio/struct.RaylibAudio.html#method.init_audio_device) —
  open the audio device; returns `Result<RaylibAudio, …>`. Panics if called twice.
- [`Wave`](https://docs.rs/raylib/latest/raylib/core/audio/struct.Wave.html) —
  loaded wave data (raw PCM). Create via `RaylibAudio::new_wave`.
- [`Sound`](https://docs.rs/raylib/latest/raylib/core/audio/struct.Sound.html) —
  short audio clip ready for playback. Create via `RaylibAudio::new_sound`.
- [`Music`](https://docs.rs/raylib/latest/raylib/core/audio/struct.Music.html) —
  audio stream for longer tracks. Create via `RaylibAudio::new_music`.
- [`AudioStream`](https://docs.rs/raylib/latest/raylib/core/audio/struct.AudioStream.html) —
  raw PCM stream for custom audio generation. Create via
  `RaylibAudio::new_audio_stream`.
- [`Sound::play`](https://docs.rs/raylib/latest/raylib/core/audio/struct.Sound.html#method.play) —
  play a `Sound` to completion (fire and forget).
- [`Music::play_stream`](https://docs.rs/raylib/latest/raylib/core/audio/struct.Music.html#method.play_stream) —
  begin streaming a `Music` track; call `update_music_stream` each frame.
- [`AudioStream::is_processed`](https://docs.rs/raylib/latest/raylib/core/audio/struct.AudioStream.html#method.is_processed) —
  check whether the stream buffer needs refilling.

## Example

```rust,no_run
# extern crate raylib;
use raylib::prelude::*;
use raylib::core::audio::RaylibAudio;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Audio demo")
        .build();

    // Open the audio device.
    let audio = RaylibAudio::init_audio_device()
        .expect("failed to init audio device");

    // Load a short sound effect; Sound is lifetime-bound to `audio`.
    let sound = audio.new_sound("assets/coin.wav")
        .expect("failed to load sound");

    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            sound.play();
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        d.draw_text("Press SPACE to play a sound", 10, 10, 20, Color::DARKGRAY);
    }
    // `sound` is dropped before `audio` — the compiler enforces this via lifetime.
    // `audio` drop calls CloseAudioDevice.
}
```

## Gotchas

- **`RaylibAudio` must outlive every audio resource.** The lifetime parameters on
  `Wave`, `Sound`, `Music`, and `AudioStream` are not just documentation — the
  compiler will reject code that drops `RaylibAudio` while any resource is still live.
- **The audio callback was removed in 6.0.** The old per-sample callback API is gone.
  Use `AudioStream` with `is_processed` / `update_audio_stream` for custom PCM
  generation.
- **WS6-prep soundness fix.** The unsound default `Sound` impl (backlog #277) was
  removed. `Sound` no longer implements `Default`.

## See also

- [RAII and resources](../core-concepts/raii-and-resources.md) — how lifetime-bound
  resources fit the broader RAII model.
- [`RaylibAudio` docs.rs](https://docs.rs/raylib/latest/raylib/core/audio/struct.RaylibAudio.html) /
  [`Sound` docs.rs](https://docs.rs/raylib/latest/raylib/core/audio/struct.Sound.html)
