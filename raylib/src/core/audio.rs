//! Contains code related to audio. [`RaylibAudio`] plays sounds and music.

use crate::{
    error::{AudioInitError, LoadSoundError, UpdateAudioStreamError},
    ffi,
};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::path::Path;

use super::error::ExportWaveError;

make_thin_wrapper_lifetime!(
    /// CPU-side waveform data loaded into RAM.
    ///
    /// A `Wave` holds raw PCM samples (and metadata) without occupying any audio-device
    /// resources. Convert it to a [`Sound`] via [`RaylibAudio::new_sound_from_wave`] for
    /// repeated low-latency playback, or export it to disk with [`Wave::export`].
    ///
    /// Freed via `UnloadWave` on drop. Lifetime is bound to the owning [`RaylibAudio`].
    Wave,
    ffi::Wave,
    RaylibAudio,
    ffi::UnloadWave
);

make_thin_wrapper_lifetime!(
    /// Audio-device-ready playable sound sample.
    ///
    /// A `Sound` is a fully decoded, GPU/audio-card-buffered sample suitable for repeated
    /// low-latency playback (e.g. effects and short clips). Load one from a file with
    /// [`RaylibAudio::new_sound`] or from a [`Wave`] with
    /// [`RaylibAudio::new_sound_from_wave`].
    ///
    /// Freed via `UnloadSound` on drop. Lifetime is bound to the owning [`RaylibAudio`].
    ///
    /// # Examples
    ///
    /// Load and play a sound effect:
    ///
    /// ```rust,no_run
    /// use raylib::prelude::*;
    /// let audio = RaylibAudio::init_audio_device().unwrap();
    /// let sound = audio.new_sound("assets/click.wav").unwrap();
    /// // Later, trigger playback:
    /// unsafe { raylib::ffi::PlaySound(*sound) };
    /// ```
    Sound,
    ffi::Sound,
    RaylibAudio,
    (ffi::UnloadSound),
    true
);
make_thin_wrapper_lifetime!(
    /// Streamed audio for long-form playback.
    ///
    /// `Music` streams data from disk (or memory) in chunks, making it suitable for
    /// background music or any audio exceeding ~10 seconds. Load via
    /// [`RaylibAudio::new_music`] and update each frame with the raylib
    /// `UpdateMusicStream` / `PlayMusicStream` calls.
    ///
    /// Freed via `UnloadMusicStream` on drop. Lifetime is bound to the owning
    /// [`RaylibAudio`].
    Music,
    ffi::Music,
    RaylibAudio,
    ffi::UnloadMusicStream
);
make_thin_wrapper_lifetime!(
    /// Low-level raw PCM streaming primitive.
    ///
    /// `AudioStream` lets you push arbitrary audio data to the audio device one
    /// buffer at a time, giving full control over sample rate, bit depth, and channel
    /// count. This is the replacement for the audio-callback API that was removed in
    /// raylib 6.0 — instead of registering a callback you periodically check
    /// `IsAudioStreamProcessed` and push the next buffer of samples.
    ///
    /// Create via [`RaylibAudio::new_audio_stream`]. Freed via `UnloadAudioStream` on
    /// drop. Lifetime is bound to the owning [`RaylibAudio`].
    ///
    /// # Examples
    ///
    /// Create a stereo 44.1 kHz stream and push silence each frame:
    ///
    /// ```rust,no_run
    /// use raylib::prelude::*;
    /// let audio = RaylibAudio::init_audio_device().unwrap();
    /// let stream = audio.new_audio_stream(44100, 16, 2);
    /// // Push PCM data when the device is ready for more samples.
    /// let silence: Vec<i16> = vec![0; 4096];
    /// unsafe { raylib::ffi::UpdateAudioStream(*stream, silence.as_ptr() as *const _, silence.len() as i32) };
    /// ```
    AudioStream,
    ffi::AudioStream,
    RaylibAudio,
    ffi::UnloadAudioStream
);

/// Owned buffer of decoded PCM samples for a [`Wave`], freed via `UnloadWaveSamples` on drop.
///
/// Returned by [`Wave::load_samples`]. Values are normalised to `[-1.0, 1.0]` and always
/// 32-bit float regardless of the source wave's `sample_size` — raylib does the conversion
/// on load. Length is `frame_count` samples (channel interleaving follows the underlying
/// wave's `channels`). Access the underlying slice via [`AsRef::<[f32]>::as_ref`]; the
/// allocation is freed when this guard drops.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let audio = RaylibAudio::init_audio_device().expect("audio init");
/// let wave = audio.new_wave("assets/click.wav").expect("wave load");
/// let samples = wave.load_samples();
/// let pcm: &[f32] = samples.as_ref();
/// println!("{} samples decoded", pcm.len());
/// // `samples` drops here → UnloadWaveSamples frees the buffer.
/// ```
///
/// # See also
///
/// - [`Wave::load_samples`] — constructor.
/// - [`Wave`] — owning wave handle whose samples this buffer decodes.
pub struct WaveSamples(*mut f32, usize);

impl AsRef<[f32]> for WaveSamples {
    fn as_ref(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self.0, self.1) }
    }
}

impl Drop for WaveSamples {
    fn drop(&mut self) {
        unsafe { ffi::UnloadWaveSamples(self.0) }
    }
}

/// A marker trait specifying an audio sample (`u8`, `i16`, or `f32`).
pub trait AudioSample: private::AudioSample {}
impl<T: private::AudioSample> AudioSample for T {}

mod private {
    pub trait AudioSample {}
    impl AudioSample for u8 {}
    impl AudioSample for i16 {}
    impl AudioSample for f32 {}
}

/// Audio subsystem handle — initializes the audio device and owns all audio resources.
///
/// `RaylibAudio` is a separate handle from [`RaylibHandle`](crate::core::RaylibHandle).
/// Obtain one with [`RaylibAudio::init_audio_device`].  All audio resource types
/// ([`Wave`], [`Sound`], [`Music`], [`AudioStream`]) are lifetime-bound to the
/// `RaylibAudio` that created them; the Rust borrow checker enforces this statically, so
/// audio resources cannot outlive the device.
///
/// The audio device is closed (`CloseAudioDevice`) when the `RaylibAudio` is dropped.
///
/// # Examples
///
/// Initialize the audio device and load a sound:
///
/// ```rust,no_run
/// use raylib::prelude::*;
/// let audio = RaylibAudio::init_audio_device().expect("audio init failed");
/// let sound = audio.new_sound("assets/click.wav").expect("sound load failed");
/// // `sound` borrows `audio` and cannot outlive it.
/// ```
#[derive(Debug, Clone)]
pub struct RaylibAudio(PhantomData<()>);

impl RaylibAudio {
    /// Initializes audio device and context.
    #[inline]
    pub fn init_audio_device() -> Result<RaylibAudio, AudioInitError> {
        unsafe {
            if ffi::IsAudioDeviceReady() {
                return Err(AudioInitError::DoubleInit);
            }
            ffi::InitAudioDevice();
            if !ffi::IsAudioDeviceReady() {
                return Err(AudioInitError::InitFailed);
            }
        }
        Ok(RaylibAudio(PhantomData))
    }

    /// Checks if audio device is ready.
    #[inline]
    #[must_use]
    pub fn is_audio_device_ready(&self) -> bool {
        unsafe { ffi::IsAudioDeviceReady() }
    }

    /// Get master volume (listener)
    #[inline]
    #[must_use]
    pub fn get_master_volume(&self) -> f32 {
        unsafe { ffi::GetMasterVolume() }
    }

    /// Sets master volume (listener).
    #[inline]
    pub fn set_master_volume(&self, volume: f32) {
        unsafe { ffi::SetMasterVolume(volume) }
    }

    /// Sets default audio buffer size for new audio streams.
    #[inline]
    pub fn set_audio_stream_buffer_size_default(&self, size: i32) {
        unsafe {
            ffi::SetAudioStreamBufferSizeDefault(size);
        }
    }

    /// Loads a new sound from file.
    #[inline]
    pub fn new_sound<'aud>(&'aud self, filename: &str) -> Result<Sound<'aud>, LoadSoundError> {
        let c_filename = CString::new(filename).unwrap();
        let s = unsafe { ffi::LoadSound(c_filename.as_ptr()) };
        if s.stream.buffer.is_null() {
            return Err(LoadSoundError::LoadFailed {
                path: filename.into(),
            });
        }

        Ok(Sound(s, self))
    }

    /// Loads sound from wave data.
    #[inline]
    pub fn new_sound_from_wave<'aud>(
        &'aud self,
        wave: &Wave,
    ) -> Result<Sound<'aud>, LoadSoundError> {
        let s = unsafe { ffi::LoadSoundFromWave(wave.0) };
        if s.stream.buffer.is_null() {
            return Err(LoadSoundError::LoadFromWaveFailed);
        }
        Ok(Sound(s, self))
    }
    /// Loads wave data from file into RAM.
    #[inline]
    pub fn new_wave<'aud>(&'aud self, filename: &str) -> Result<Wave<'aud>, LoadSoundError> {
        let c_filename = CString::new(filename).unwrap();
        let w = unsafe { ffi::LoadWave(c_filename.as_ptr()) };
        if w.data.is_null() {
            return Err(LoadSoundError::LoadWaveFromFileFailed {
                path: filename.into(),
            });
        }
        Ok(Wave(w, self))
    }

    /// Load wave from memory buffer, fileType refers to extension: i.e. '.wav'
    #[inline]
    pub fn new_wave_from_memory<'aud>(
        &'aud self,
        filetype: &str,
        bytes: &[u8],
    ) -> Result<Wave<'aud>, LoadSoundError> {
        let c_filetype = CString::new(filetype).unwrap();
        let w = unsafe {
            ffi::LoadWaveFromMemory(c_filetype.as_ptr(), bytes.as_ptr(), bytes.len() as i32)
        };
        if w.data.is_null() {
            return Err(LoadSoundError::Null);
        };
        Ok(Wave(w, self))
    }

    /// Loads music stream from file.
    #[inline]
    pub fn new_music<'aud>(&'aud self, filename: &str) -> Result<Music<'aud>, LoadSoundError> {
        let c_filename = CString::new(filename).unwrap();
        let m = unsafe { ffi::LoadMusicStream(c_filename.as_ptr()) };
        if m.stream.buffer.is_null() {
            return Err(LoadSoundError::LoadMusicFromFileFailed {
                path: filename.into(),
            });
        }
        Ok(Music(m, self))
    }

    /// Load music stream from data
    #[inline]
    pub fn new_music_from_memory<'aud>(
        &'aud self,
        filetype: &str,
        bytes: &[u8],
    ) -> Result<Music<'aud>, LoadSoundError> {
        let c_filetype = CString::new(filetype).unwrap();
        let w = unsafe {
            ffi::LoadMusicStreamFromMemory(c_filetype.as_ptr(), bytes.as_ptr(), bytes.len() as i32)
        };
        if w.stream.buffer.is_null() {
            return Err(LoadSoundError::MusicNull);
        };
        Ok(Music(w, self))
    }

    /// Initializes audio stream (to stream raw PCM data).
    #[inline]
    #[must_use]
    pub fn new_audio_stream(
        &self,
        sample_rate: u32,
        sample_size: u32,
        channels: u32,
    ) -> AudioStream<'_> {
        unsafe {
            AudioStream(
                ffi::LoadAudioStream(sample_rate, sample_size, channels),
                self,
            )
        }
    }
}

impl Drop for RaylibAudio {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::CloseAudioDevice() }
    }
}

impl Wave<'_> {
    /// Total number of frames (considering channels)
    #[inline]
    #[must_use]
    pub const fn frame_count(&self) -> u32 {
        self.0.frameCount
    }
    /// Frequency (samples per second)
    #[inline]
    #[must_use]
    pub const fn sample_rate(&self) -> u32 {
        self.0.sampleRate
    }
    /// Bit depth (bits per sample): 8, 16, 32 (24 not supported)
    #[inline]
    #[must_use]
    pub const fn sample_size(&self) -> u32 {
        self.0.sampleSize
    }
    /// Number of channels (1-mono, 2-stereo, ...)
    #[inline]
    #[must_use]
    pub const fn channels(&self) -> u32 {
        self.0.channels
    }
    /// # Safety
    ///
    /// Caller takes ownership of the raw wave data and must free it via `UnloadWave`.
    #[inline]
    #[must_use]
    pub unsafe fn inner(self) -> ffi::Wave {
        let inner = self.0;
        std::mem::forget(self);
        inner
    }

    /// Checks if wave data is valid (data loaded and parameters)
    #[inline]
    #[must_use]
    pub fn is_wave_valid(&self) -> bool {
        unsafe { ffi::IsWaveValid(self.0) }
    }

    /// Export wave file. Extension must be .wav or .raw
    #[inline]
    pub fn export(&self, filename: impl AsRef<Path>) -> Result<(), ExportWaveError> {
        let c_filename = CString::new(filename.as_ref().to_string_lossy().as_bytes()).unwrap();
        let success = unsafe { ffi::ExportWave(self.0, c_filename.as_ptr()) };
        if success {
            Ok(())
        } else {
            // const WAV: &CStr = c".wav";
            const QOA: &CStr = c".qoa";
            // const RAW: &CStr = c".raw";
            let is_qoa = unsafe { ffi::IsFileExtension(c_filename.as_ptr(), QOA.as_ptr()) };
            if is_qoa {
                let samples = self.0.sampleSize as i32;
                if samples != 16 {
                    return Err(ExportWaveError::QoaBadSamples(self.0.sampleSize as i32));
                }
            }
            Err(ExportWaveError::ExportFailed)
        }
    }

    /*/// Export wave sample data to code (.h)
    #[inline]
    pub fn export_wave_as_code(&self, filename: &str) -> bool {
        let c_filename = CString::new(filename).unwrap();
        unsafe { ffi::ExportWaveAsCode(self.0, c_filename.as_ptr()) }
    }*/

    /// Copies a wave to a new wave.
    #[inline]
    #[must_use]
    pub(crate) fn copy(&'_ self) -> Wave<'_> {
        unsafe { Wave(ffi::WaveCopy(self.0), self.1) }
    }

    /// Converts wave data to desired format.
    #[inline]
    pub fn format(&mut self, sample_rate: i32, sample_size: i32, channels: i32) {
        unsafe { ffi::WaveFormat(&mut self.0, sample_rate, sample_size, channels) }
    }

    /// Crops a wave to defined sample range.
    #[inline]
    pub fn crop(&mut self, init_sample: i32, final_sample: i32) {
        unsafe { ffi::WaveCrop(&mut self.0, init_sample, final_sample) }
    }

    /// Load samples data from wave as a floats array
    /// NOTE 1: Returned sample values are normalized to range [-1..1]
    /// NOTE 2: Sample data allocated should be freed with UnloadWaveSamples()
    #[inline]
    #[must_use]
    pub fn load_samples(&self) -> WaveSamples {
        WaveSamples(
            unsafe { ffi::LoadWaveSamples(self.0) },
            self.frameCount as usize,
        )
    }
}

impl Sound<'_> {
    /// Checks if a sound is valid (data loaded and buffers initialized)
    #[inline]
    #[must_use]
    pub fn is_sound_valid(&self) -> bool {
        unsafe { ffi::IsSoundValid(self.0) }
    }

    /// Total number of frames (considering channels)
    #[inline]
    #[must_use]
    pub const fn frame_count(&self) -> u32 {
        self.0.frameCount
    }
    /// # Safety
    ///
    /// Caller takes ownership of the raw sound data and must free it via `UnloadSound`.
    #[inline]
    #[must_use]
    pub unsafe fn inner(self) -> ffi::Sound {
        let inner = self.0;
        std::mem::forget(self);
        inner
    }

    /// Plays a sound.
    #[inline]
    pub fn play(&self) {
        unsafe { ffi::PlaySound(self.0) }
    }

    /// Pauses a sound.
    #[inline]
    pub fn pause(&self) {
        unsafe { ffi::PauseSound(self.0) }
    }

    /// Resumes a paused sound.
    #[inline]
    pub fn resume(&self) {
        unsafe { ffi::ResumeSound(self.0) }
    }

    /// Stops playing a sound.
    #[inline]
    pub fn stop(&self) {
        unsafe { ffi::StopSound(self.0) }
    }

    /// Checks if a sound is currently playing.
    #[inline]
    #[must_use]
    pub fn is_playing(&self) -> bool {
        unsafe { ffi::IsSoundPlaying(self.0) }
    }

    /// Sets volume for a sound (`1.0` is max level).
    #[inline]
    pub fn set_volume(&self, volume: f32) {
        unsafe { ffi::SetSoundVolume(self.0, volume) }
    }

    /// Sets pitch for a sound (`1.0` is base level).
    #[inline]
    pub fn set_pitch(&self, pitch: f32) {
        unsafe { ffi::SetSoundPitch(self.0, pitch) }
    }

    /// Set pan for a sound (0.5 is center)
    #[inline]
    pub fn set_pan(&self, pan: f32) {
        unsafe { ffi::SetSoundPan(self.0, pan) }
    }

    /// Updates sound buffer with new data.
    /// **Notes** (iann):
    /// 1. raylib’s `UpdateSound` is a raw `memcpy` without size checks, we add safety checks to  here to prevent invalid memory writes.
    ///     - potential upstream raylib discussion: "too many frames" doesn't exist for the `Sound`'s `AudioStream`
    ///     - potential upstream raylib discussion: adding sampleSize checks for the `memcpy`
    /// 2. raylib's `Sound`'s `AudioStream` always gets 32-bit sample size (so we always catch non-32-bit `Sound`'s with a `SampleSizeMismatch`)
    ///     - 32-bit fixed in config here: <https://github.com/raysan5/raylib/blob/master/src/config.h#L282>
    ///     - device format set here: <https://github.com/raysan5/raylib/blob/master/src/raudio.c#L288>
    ///     - potential upstream raylib discussion: allowing for other samplesSizes for `Sound`
    #[inline]
    pub fn update<T: AudioSample>(&mut self, data: &[T]) -> Result<(), UpdateAudioStreamError> {
        let expected_sample_size_bits =
            usize::try_from(self.stream.sampleSize).expect("sampleSize should be 8, 16, or 32");
        let provided_sample_size_bits = size_of::<T>() * u8::BITS as usize;
        if provided_sample_size_bits != expected_sample_size_bits {
            return Err(UpdateAudioStreamError::SampleSizeMismatch {
                expected: expected_sample_size_bits,
                provided: provided_sample_size_bits,
            });
        }
        let max_frame_count = usize::try_from(self.frameCount)
            .expect("frameCount should be a valid memory allocation size");
        let provided_frame_count = data.len();
        if provided_frame_count > max_frame_count {
            return Err(UpdateAudioStreamError::TooManyFrames {
                max: max_frame_count,
                provided: provided_frame_count,
            });
        }
        unsafe {
            ffi::UpdateSound(
                self.0,
                data.as_ptr() as *const std::os::raw::c_void,
                provided_frame_count.try_into().unwrap(),
            );
        }
        Ok(())
    }
}

impl SoundAlias<'_, '_> {
    /// Checks if a sound is valid (data loaded and buffers initialized)
    #[inline]
    #[must_use]
    pub fn is_sound_valid(&self) -> bool {
        unsafe { ffi::IsSoundValid(self.0) }
    }

    /// Total number of frames (considering channels)
    #[inline]
    #[must_use]
    pub const fn frame_count(&self) -> u32 {
        self.0.frameCount
    }
    /// # Safety
    ///
    /// Caller takes ownership of the raw sound alias data and must free it via `UnloadSoundAlias`.
    #[must_use]
    pub unsafe fn inner(self) -> ffi::Sound {
        let inner = self.0;
        std::mem::forget(self);
        inner
    }

    /// Plays a sound.
    #[inline]
    pub fn play(&self) {
        unsafe { ffi::PlaySound(self.0) }
    }

    /// Pauses a sound.
    #[inline]
    pub fn pause(&self) {
        unsafe { ffi::PauseSound(self.0) }
    }

    /// Resumes a paused sound.
    #[inline]
    pub fn resume(&self) {
        unsafe { ffi::ResumeSound(self.0) }
    }

    /// Stops playing a sound.
    #[inline]
    pub fn stop(&self) {
        unsafe { ffi::StopSound(self.0) }
    }

    /// Checks if a sound is currently playing.
    #[inline]
    #[must_use]
    pub fn is_playing(&self) -> bool {
        unsafe { ffi::IsSoundPlaying(self.0) }
    }

    /// Sets volume for a sound (`1.0` is max level).
    #[inline]
    pub fn set_volume(&self, volume: f32) {
        unsafe { ffi::SetSoundVolume(self.0, volume) }
    }

    /// Sets pitch for a sound (`1.0` is base level).
    #[inline]
    pub fn set_pitch(&self, pitch: f32) {
        unsafe { ffi::SetSoundPitch(self.0, pitch) }
    }

    /// Set pan for a sound (0.5 is center)
    #[inline]
    pub fn set_pan(&self, pan: f32) {
        unsafe { ffi::SetSoundPan(self.0, pan) }
    }
}

impl Drop for SoundAlias<'_, '_> {
    fn drop(&mut self) {
        unsafe { ffi::UnloadSoundAlias(self.0) }
    }
}

impl Music<'_> {
    /// Starts music playing.
    #[inline]
    pub fn play_stream(&self) {
        unsafe { ffi::PlayMusicStream(self.0) }
    }

    /// Updates buffers for music streaming.
    #[inline]
    pub fn update_stream(&self) {
        unsafe { ffi::UpdateMusicStream(self.0) }
    }

    /// Stops music playing.
    #[inline]
    pub fn stop_stream(&self) {
        unsafe { ffi::StopMusicStream(self.0) }
    }

    /// Pauses music playing.
    #[inline]
    pub fn pause_stream(&self) {
        unsafe { ffi::PauseMusicStream(self.0) }
    }

    /// Resumes playing paused music.
    #[inline]
    pub fn resume_stream(&self) {
        unsafe { ffi::ResumeMusicStream(self.0) }
    }

    /// Checks if music is playing.
    #[inline]
    #[must_use]
    pub fn is_stream_playing(&self) -> bool {
        unsafe { ffi::IsMusicStreamPlaying(self.0) }
    }

    /// Sets volume for music (`1.0` is max level).
    #[inline]
    pub fn set_volume(&self, volume: f32) {
        unsafe { ffi::SetMusicVolume(self.0, volume) }
    }

    /// Sets pitch for music (`1.0` is base level).
    #[inline]
    pub fn set_pitch(&self, pitch: f32) {
        unsafe { ffi::SetMusicPitch(self.0, pitch) }
    }

    /// Gets music time length in seconds.
    #[inline]
    #[must_use]
    pub fn get_time_length(&self) -> f32 {
        unsafe { ffi::GetMusicTimeLength(self.0) }
    }

    /// Gets current music time played in seconds.
    #[inline]
    #[must_use]
    pub fn get_time_played(&self) -> f32 {
        unsafe { ffi::GetMusicTimePlayed(self.0) }
    }

    /// Seek music to a position (in seconds)
    #[inline]
    pub fn seek_stream(&self, position: f32) {
        unsafe { ffi::SeekMusicStream(self.0, position) }
    }

    /// Set pan for a music (0.5 is center)
    #[inline]
    pub fn set_pan(&self, pan: f32) {
        unsafe { ffi::SetMusicPan(self.0, pan) }
    }

    /// Checks if a music stream is valid (context and buffers initialized)
    #[inline]
    #[must_use]
    pub fn is_music_valid(&self) -> bool {
        unsafe { ffi::IsMusicValid(self.0) }
    }
}

impl AudioStream<'_> {
    /// Checks if an audio stream is valid (buffers initialized)
    #[inline]
    #[must_use]
    pub fn is_audio_stream_valid(&self) -> bool {
        unsafe { ffi::IsAudioStreamValid(self.0) }
    }
    /// Frequency (samples per second)
    #[inline]
    #[must_use]
    pub const fn sample_rate(&self) -> u32 {
        self.0.sampleRate
    }
    /// Bit depth (bits per sample): 8, 16, 32 (24 not supported)
    #[inline]
    #[must_use]
    pub const fn sample_size(&self) -> u32 {
        self.0.sampleSize
    }
    /// Number of channels (1-mono, 2-stereo, ...)
    #[inline]
    #[must_use]
    pub const fn channels(&self) -> u32 {
        self.0.channels
    }

    /// # Safety
    ///
    /// Caller takes ownership of the raw audio stream and must free it via `UnloadAudioStream`.
    #[must_use]
    pub unsafe fn inner(self) -> ffi::AudioStream {
        let inner = self.0;
        std::mem::forget(self);
        inner
    }

    /// Updates audio stream buffers with data.
    #[inline]
    pub fn update<T: AudioSample>(&mut self, data: &[T]) -> Result<(), UpdateAudioStreamError> {
        let expected_sample_size =
            usize::try_from(self.sampleSize).expect("sampleSize should be 8, 16, or 32");
        let provided_sample_size_bits = size_of::<T>() * u8::BITS as usize;
        if provided_sample_size_bits != expected_sample_size {
            return Err(UpdateAudioStreamError::SampleSizeMismatch {
                expected: expected_sample_size,
                provided: provided_sample_size_bits,
            });
        }
        let provided_frame_count = data.len();

        unsafe {
            ffi::UpdateAudioStream(
                self.0,
                data.as_ptr() as *const std::os::raw::c_void,
                provided_frame_count.try_into().unwrap(),
            );
        }
        Ok(())
    }

    /// Plays audio stream.
    #[inline]
    pub fn play(&self) {
        unsafe {
            ffi::PlayAudioStream(self.0);
        }
    }

    /// Pauses audio stream.
    #[inline]
    pub fn pause(&self) {
        unsafe {
            ffi::PauseAudioStream(self.0);
        }
    }

    /// Resumes audio stream.
    #[inline]
    pub fn resume(&self) {
        unsafe {
            ffi::ResumeAudioStream(self.0);
        }
    }

    /// Checks if audio stream is currently playing.
    #[inline]
    #[must_use]
    pub fn is_playing(&self) -> bool {
        unsafe { ffi::IsAudioStreamPlaying(self.0) }
    }

    /// Stops audio stream.
    #[inline]
    pub fn stop(&self) {
        unsafe {
            ffi::StopAudioStream(self.0);
        }
    }

    /// Sets volume for audio stream (`1.0` is max level).
    #[inline]
    pub fn set_volume(&self, volume: f32) {
        unsafe {
            ffi::SetAudioStreamVolume(self.0, volume);
        }
    }

    /// Sets pitch for audio stream (`1.0` is base level).
    #[inline]
    pub fn set_pitch(&self, pitch: f32) {
        unsafe {
            ffi::SetAudioStreamPitch(self.0, pitch);
        }
    }

    /// Sets pitch for audio stream (`1.0` is base level).
    #[inline]
    #[must_use]
    pub fn is_processed(&self) -> bool {
        unsafe { ffi::IsAudioStreamProcessed(self.0) }
    }

    /// Set pan for audio stream (0.5 is centered)
    #[inline]
    pub fn set_pan(&self, pan: f32) {
        unsafe {
            ffi::SetAudioStreamPan(self.0, pan);
        }
    }
}

impl<'bind> Sound<'bind> {
    /// Clone sound from existing sound data, clone does not own wave data
    // NOTE: Wave data must be unallocated manually and will be shared across all clones
    pub fn alias<'snd>(&'snd self) -> Result<SoundAlias<'snd, 'bind>, LoadSoundError> {
        let s = unsafe { ffi::LoadSoundAlias(self.0) };
        if s.stream.buffer.is_null() {
            return Err(LoadSoundError::LoadFromWaveFailed);
        }
        Ok(SoundAlias(s, PhantomData))
    }
}

/// A lightweight alias handle to a [`Sound`] that shares the same audio buffer without owning it.
///
/// Returned by [`Sound::alias`]. Wraps `LoadSoundAlias` — the source [`Sound`] owns the audio
/// buffer and remains responsible for unloading it; the alias holds an independent playback
/// head (volume, pitch, pan, play position) so multiple overlapping instances of the same
/// sound can play concurrently without allocating a fresh decoded buffer per voice.
///
/// The `'snd` lifetime borrows the parent [`Sound`] (statically enforced) so the alias
/// cannot outlive the buffer it points at; the `'bind` lifetime tracks the parent's audio
/// device.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let audio = RaylibAudio::init_audio_device().expect("audio init");
/// let sound = audio.new_sound("assets/click.wav").expect("sound load");
/// let voice_a = sound.alias().expect("alias");
/// let voice_b = sound.alias().expect("alias");
/// // `voice_a` and `voice_b` can be played simultaneously and tuned independently.
/// ```
///
/// # See also
///
/// - [`Sound::alias`] — constructor.
/// - [`Sound`] — owning sound handle whose buffer this alias shares.
pub struct SoundAlias<'snd, 'bind>(ffi::Sound, PhantomData<&'snd Sound<'bind>>);
