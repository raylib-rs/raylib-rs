//! Definitions for error types used throughout the crate

use thiserror::Error;

/// Errors returned when initializing the raylib audio device.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::audio::RaylibAudio;
/// use raylib::core::error::AudioInitError;
///
/// match RaylibAudio::init_audio_device() {
///     Ok(_audio) => { /* use audio */ }
///     Err(AudioInitError::DoubleInit) => {
///         eprintln!("audio device is already initialized");
///     }
///     Err(AudioInitError::InitFailed) => {
///         eprintln!("audio device backend rejected initialization");
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum AudioInitError {
    /// The audio device has already been initialized by a live `RaylibAudio` instance.
    ///
    /// **Cause:** [`ffi::IsAudioDeviceReady`](crate::ffi::IsAudioDeviceReady) returned true
    /// before [`ffi::InitAudioDevice`](crate::ffi::InitAudioDevice) was called, meaning a
    /// previous `RaylibAudio` handle is still alive.
    ///
    /// **Recovery:** Drop the existing `RaylibAudio` handle before initializing a new one.
    /// Only one audio device may be active at a time.
    #[error("RaylibAudio cannot be instantiated more then once at a time")]
    DoubleInit,
    /// The audio backend reported failure when bringing the device online.
    ///
    /// **Cause:** After [`ffi::InitAudioDevice`](crate::ffi::InitAudioDevice) ran,
    /// [`ffi::IsAudioDeviceReady`](crate::ffi::IsAudioDeviceReady) still returned false —
    /// usually missing audio drivers, a busy device, or a permissions failure.
    ///
    /// **Recovery:** Check OS audio configuration and permissions; on headless CI you may
    /// need a dummy audio device. No retry will succeed without a backend change.
    #[error("failed to initialize audio device")]
    InitFailed,
}

/// Errors returned when exporting wave data to a file.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::ExportWaveError;
///
/// fn handle(e: ExportWaveError) {
///     match e {
///         ExportWaveError::QoaBadSamples(bits) => {
///             eprintln!("QOA requires 16-bit samples (got {bits})");
///         }
///         ExportWaveError::ExportFailed => {
///             eprintln!("raylib refused to write the wave file");
///         }
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum ExportWaveError {
    /// The wave's sample depth is not 16 bits, which is the only depth QOA encoding accepts.
    ///
    /// **Cause:** The caller asked to export as QOA but the [`ffi::Wave`](crate::ffi::Wave)'s
    /// `sampleSize` field is not 16. raylib's QOA exporter only handles 16-bit PCM.
    ///
    /// **Recovery:** Convert the wave to 16-bit (e.g. via [`ffi::WaveFormat`](crate::ffi::WaveFormat))
    /// before exporting, or choose a different output format such as WAV.
    #[error("wave data must be 16 bit per sample for QOA format export (actual: {0})")]
    QoaBadSamples(i32),
    /// raylib's [`ffi::ExportWave`](crate::ffi::ExportWave) reported failure when writing the file.
    ///
    /// **Cause:** The underlying file write failed — usually a missing parent directory,
    /// unwritable path, or an unsupported file extension.
    ///
    /// **Recovery:** Verify the path is writable, the parent directory exists, and the
    /// extension matches one of raylib's supported wave formats (`.wav`, `.qoa`, `.raw`).
    #[error("failed to export wave data")]
    ExportFailed,
}

/// Errors returned when loading a `Sound` or `Music` resource.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::LoadSoundError;
///
/// fn handle(e: LoadSoundError) {
///     match e {
///         LoadSoundError::LoadFailed { path } => eprintln!("sound load failed: {path}"),
///         LoadSoundError::LoadFromWaveFailed => eprintln!("sound from wave failed"),
///         LoadSoundError::LoadWaveFromFileFailed { path } => eprintln!("wave load failed: {path}"),
///         LoadSoundError::Null => eprintln!("wave buffer was null"),
///         LoadSoundError::LoadMusicFromFileFailed { path } => eprintln!("music load failed: {path}"),
///         LoadSoundError::MusicNull => eprintln!("music buffer was null"),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum LoadSoundError {
    /// raylib's [`ffi::LoadSound`](crate::ffi::LoadSound) did not return a ready sound.
    ///
    /// **Cause:** The audio file at `path` was missing, unreadable, or in an unsupported
    /// format; the resulting [`ffi::Sound`](crate::ffi::Sound) reported a null buffer.
    ///
    /// **Recovery:** Confirm the file exists (`std::fs::metadata(&path)`) and that its
    /// extension is one of raylib's supported audio formats (`.wav`, `.ogg`, `.mp3`,
    /// `.flac`, `.qoa`, `.xm`, `.mod`).
    #[error("failed to load sound\npath: {path:?}")]
    LoadFailed {
        /// Path to the audio file raylib was unable to load.
        path: String,
    },
    /// Loading a sound from an in-memory [`ffi::Wave`](crate::ffi::Wave) produced an empty sound.
    ///
    /// **Cause:** [`ffi::LoadSoundFromWave`](crate::ffi::LoadSoundFromWave) returned a sound
    /// with a null backing buffer, usually because the source `Wave` was itself invalid or empty.
    ///
    /// **Recovery:** Validate the source `Wave` (non-zero frame count, non-null `data` pointer)
    /// before converting to `Sound`.
    #[error("failed to load sound from wave")]
    LoadFromWaveFailed,
    /// raylib's [`ffi::LoadWave`](crate::ffi::LoadWave) did not return a ready wave.
    ///
    /// **Cause:** The file at `path` was missing, unreadable, or in a format raylib's wave
    /// decoder does not support; the resulting `Wave` had a null `data` pointer.
    ///
    /// **Recovery:** Verify the path exists and uses one of `.wav`, `.ogg`, `.mp3`, `.flac`,
    /// or `.qoa`. Convert other formats up front.
    #[error("cannot load wave\npath: {path:?}")]
    LoadWaveFromFileFailed {
        /// Path to the wave file raylib was unable to decode.
        path: String,
    },
    /// raylib returned a [`ffi::Wave`](crate::ffi::Wave) whose `data` pointer is null.
    ///
    /// **Cause:** The buffer that was supposed to back the wave came back as null, indicating
    /// either an allocation failure or an empty source.
    ///
    /// **Recovery:** Check the source buffer's length and contents; retry with a non-empty,
    /// well-formed payload.
    #[error("wave data is null, check provided buffer data")]
    Null,
    /// raylib's [`ffi::LoadMusicStream`](crate::ffi::LoadMusicStream) did not return a ready stream.
    ///
    /// **Cause:** The music file at `path` was missing, unreadable, or in an unsupported
    /// format; the resulting [`ffi::Music`](crate::ffi::Music) reported a null buffer.
    ///
    /// **Recovery:** Confirm the file exists and uses one of raylib's supported music
    /// formats; large streaming formats (`.mp3`, `.ogg`, `.flac`) usually work.
    #[error("music could not be loaded from file\npath: {path:?}")]
    LoadMusicFromFileFailed {
        /// Path to the music file raylib was unable to open.
        path: String,
    },
    /// raylib returned an [`ffi::Music`](crate::ffi::Music) whose backing buffer pointer is null.
    ///
    /// **Cause:** The streaming buffer raylib allocated for the music came back as null,
    /// indicating either an allocation failure or a corrupt source.
    ///
    /// **Recovery:** Validate the source buffer and retry; if the source is in-memory data,
    /// confirm the format hint matches the bytes provided.
    #[error("music's buffer data data is null, check provided buffer data")]
    MusicNull,
}

/// Errors that can occur when pushing new audio data into a `Sound` or `AudioStream`.
/// **Notes** (iann): if raylib upstream discussion introduces any of these checks, we might simplify these to avoid any redundancy i think
/// 1. `SampleSizeMismatch` is raylib-rs only, raylib does not do sampleSize matching checks.
/// 2. `TooManyFrames` comes from the WARNING behavior in raylib `UpdateAudioStreamInLockedState`: <https://github.com/raysan5/raylib/blob/master/src/raudio.c#L2662>
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::UpdateAudioStreamError;
///
/// fn handle(e: UpdateAudioStreamError) {
///     match e {
///         UpdateAudioStreamError::SampleSizeMismatch { expected, provided } => {
///             eprintln!("stream wants {expected}-bit samples, got {provided}-bit");
///         }
///         UpdateAudioStreamError::TooManyFrames { max, provided } => {
///             eprintln!("buffer holds {max} frames, caller tried to write {provided}");
///         }
///         UpdateAudioStreamError::CallbackSlotBusy => {
///             eprintln!("clear the existing callback first");
///         }
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum UpdateAudioStreamError {
    /// The provided sample type's bit depth does not match the stream's configured `sampleSize`.
    ///
    /// **Cause:** raylib-rs compared `size_of::<T>() * 8` against the audio stream's
    /// `sampleSize` field and they disagreed. raylib itself does not perform this check;
    /// it would `memcpy` silently and produce garbled audio.
    ///
    /// **Recovery:** Pass samples of the type the stream was created with (typically `i16`
    /// for 16-bit streams or `f32` for 32-bit streams, which is the default for `Sound`).
    #[error("update data format must match sound: expected {expected} bits, got {provided} bits")]
    SampleSizeMismatch {
        /// Sample bit depth the audio stream was configured with.
        expected: usize,
        /// Sample bit depth derived from the type the caller passed in.
        provided: usize,
    },
    /// The caller asked to write more frames than the audio stream's buffer can hold.
    ///
    /// **Cause:** raylib's `UpdateAudioStreamInLockedState` logs a WARNING and truncates;
    /// raylib-rs rejects the call instead so the caller does not silently lose frames.
    ///
    /// **Recovery:** Split the data into chunks no larger than `max` frames, or grow the
    /// stream's buffer size at creation time.
    #[error("Attempting to write too many frames to buffer: provided {provided}, max {max}")]
    TooManyFrames {
        /// Maximum number of frames the stream's internal buffer can accept in one write.
        max: usize,
        /// Number of frames the caller attempted to write.
        provided: usize,
    },
    /// The audio stream already has a callback installed in the requested slot.
    ///
    /// **Cause:** raylib-rs tracks ownership of the stream's processor callback and refuses
    /// to overwrite a live one.
    ///
    /// **Recovery:** Call `unset_audio_stream_callback()` (or drop the stream) before
    /// installing a new callback.
    #[error(
        "AudioStream's callback slot is already in use; call unset_audio_stream_callback() to clear it"
    )]
    CallbackSlotBusy,
}

/// Errors returned when a memory allocation via raylib's allocator fails.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::AllocationError;
///
/// fn handle(e: AllocationError) {
///     match e {
///         AllocationError::NullAlloc => eprintln!("allocator returned null"),
///         AllocationError::IntoUIntFailed => eprintln!("requested allocation too large"),
///         AllocationError::ZeroBytes => eprintln!("zero-byte allocation requested"),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum AllocationError {
    /// [`MemAlloc`](crate::ffi::MemAlloc) returned null.
    ///
    /// **Cause:** raylib's internal allocator (or any custom allocator registered via
    /// [`SetTraceLogCallback`](crate::ffi::SetTraceLogCallback)'s family) returned a null
    /// pointer for the requested byte count — typically out-of-memory at the process or
    /// platform level.
    ///
    /// **Recovery:** Free unused resources (drop unused `Image`/`Mesh`/`Texture` handles)
    /// and retry, or report unrecoverable allocation failure to the caller.
    #[error("memory request exceeds capacity")]
    NullAlloc,
    /// The size of `[T; count]` in bytes exceeds [`u32::MAX`]
    /// (the largest value [`MemAlloc`](crate::ffi::MemAlloc) can be passed).
    ///
    /// **Cause:** The caller asked to allocate a slice whose total byte size exceeds
    /// `u32::MAX` (~4 GiB). raylib's `MemAlloc` accepts `unsigned int` so larger
    /// requests cannot be expressed through the FFI.
    ///
    /// **Recovery:** Split the allocation into smaller chunks (e.g. stream image data
    /// in tiles), or use a Rust-side allocator (`Box`/`Vec`) for buffers raylib does
    /// not need to own.
    #[error("memory request in bytes exceeds unsigned integer maximum")]
    IntoUIntFailed,
    /// Attempted to pass 0 to [`MemAlloc`](crate::ffi::MemAlloc).
    ///
    /// **Cause:** Zero-byte allocations are not meaningful and raylib's allocator
    /// rejects them; raylib-rs catches the request before the FFI call.
    ///
    /// **Recovery:** Skip the call entirely for empty containers, or use a sentinel
    /// (e.g. `Option<Box<[T]>>`) to represent "no allocation needed".
    #[error("requested zero bytes of memory")]
    ZeroBytes,
}

/// Errors returned when validating a `Mesh` before upload or generation.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::InvalidMeshError;
///
/// fn handle(e: InvalidMeshError) {
///     match e {
///         InvalidMeshError::TrianglePointMiscount => eprintln!("vertex/index count is not a multiple of 3"),
///         InvalidMeshError::IndexOutOfBounds => eprintln!("an index refers to a vertex past the end of the buffer"),
///         InvalidMeshError::VertexUnindexible(_) => eprintln!("more than u16::MAX vertices"),
///         InvalidMeshError::TexcoordsMiscount => eprintln!("texcoord count != vertex count"),
///         InvalidMeshError::Texcoords2Miscount => eprintln!("texcoords2 count != vertex count"),
///         InvalidMeshError::NormalsMiscount => eprintln!("normal count != vertex count"),
///         InvalidMeshError::TangentsMiscount => eprintln!("tangent count != vertex count"),
///         InvalidMeshError::ColorsMiscount => eprintln!("color count != vertex count"),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum InvalidMeshError {
    /// The mesh's vertex or index count is not divisible by three.
    ///
    /// **Cause:** raylib expects triangle lists, so the buffer length modulo three must be
    /// zero. Indexed meshes must have an index count divisible by three; non-indexed meshes
    /// must have a vertex count divisible by three.
    ///
    /// **Recovery:** Pad or trim the buffer so its length is a multiple of three, or fix
    /// the geometry generator that produced the wrong count.
    #[error("mesh should have 3 indices/vertices for each triangle")]
    TrianglePointMiscount,
    /// One or more indices reference a vertex position past the end of the vertex buffer.
    ///
    /// **Cause:** An entry in the indices slice is `>= vertex_count`, so dereferencing it
    /// at draw time would read past the vertex buffer.
    ///
    /// **Recovery:** Clamp or regenerate the indices so every value is in `0..vertex_count`.
    #[error("indices should be within the number of vertices")]
    IndexOutOfBounds,
    /// The vertex count cannot be represented as `u16`, so the mesh cannot be indexed.
    ///
    /// **Cause:** raylib's index buffer uses `u16`, so meshes with more than `u16::MAX`
    /// vertices cannot be referenced by index.
    ///
    /// **Recovery:** Split the mesh into multiple sub-meshes, each with at most `u16::MAX`
    /// vertices, or upload a non-indexed mesh.
    #[error("mesh with indices should not exceed u16::MAX vertices")]
    VertexUnindexible(std::num::TryFromIntError),
    /// The primary texcoord buffer length does not match the vertex count.
    ///
    /// **Cause:** raylib expects one `Vector2` UV per vertex; the caller supplied a
    /// different count.
    ///
    /// **Recovery:** Resize the texcoord buffer to match the vertex count, or omit the
    /// buffer entirely (`None`) if UVs are not needed.
    #[error("mesh should have one texcoord per vertex")]
    TexcoordsMiscount,
    /// The secondary texcoord buffer length does not match the vertex count.
    ///
    /// **Cause:** The optional `texcoords2` slice was supplied with a length different from
    /// the vertex count.
    ///
    /// **Recovery:** Resize `texcoords2` to match the vertex count, or pass `None` to
    /// disable the secondary UV channel.
    #[error("mesh with texcoords2 should have one per vertex")]
    Texcoords2Miscount,
    /// The normals buffer length does not match the vertex count.
    ///
    /// **Cause:** raylib expects one normal per vertex; the slice supplied was a different
    /// length.
    ///
    /// **Recovery:** Resize the normals buffer to match the vertex count, or pass `None`
    /// to skip per-vertex normals.
    #[error("mesh with normals should have one per vertex")]
    NormalsMiscount,
    /// The tangents buffer length does not match the vertex count.
    ///
    /// **Cause:** raylib expects one tangent per vertex; the slice supplied was a different
    /// length.
    ///
    /// **Recovery:** Resize the tangents buffer to match the vertex count, or pass `None`
    /// to skip per-vertex tangents.
    #[error("mesh with tangents should have one per vertex")]
    TangentsMiscount,
    /// The colors buffer length does not match the vertex count.
    ///
    /// **Cause:** raylib expects one color per vertex; the slice supplied was a different
    /// length.
    ///
    /// **Recovery:** Resize the colors buffer to match the vertex count, or pass `None`
    /// to skip per-vertex colors.
    #[error("mesh with colors should have one per vertex")]
    ColorsMiscount,
}

/// Errors returned when procedurally generating a mesh.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::GenMeshError;
///
/// fn handle(e: GenMeshError) {
///     match e {
///         GenMeshError::InvalidMesh(inner) => eprintln!("mesh validation failed: {inner}"),
///         GenMeshError::Allocation(inner) => eprintln!("mesh allocation failed: {inner}"),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum GenMeshError {
    /// The caller-supplied mesh data did not pass [`InvalidMeshError`] validation.
    ///
    /// **Cause:** One of the per-buffer length or index-range checks in [`InvalidMeshError`]
    /// returned a concrete failure; this variant transports that inner error.
    ///
    /// **Recovery:** Inspect the inner [`InvalidMeshError`] for the specific buffer that
    /// failed and fix the offending input.
    #[error("provided mesh data does not correspond to a valid mesh")]
    InvalidMesh(#[from] InvalidMeshError),
    /// raylib's allocator failed while building the mesh's GPU-side buffers.
    ///
    /// **Cause:** [`ffi::MemAlloc`](crate::ffi::MemAlloc) returned null or rejected the
    /// requested byte count when raylib-rs tried to copy mesh data into raylib-owned storage.
    ///
    /// **Recovery:** Reduce the mesh size, free unused resources, or inspect the inner
    /// [`AllocationError`] for the specific failure mode.
    #[error("could not allocate memory for the mesh data")]
    Allocation(#[from] AllocationError),
}

/// Errors returned when compressing data.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::CompressionError;
///
/// match raylib::core::data::compress_data(b"hello") {
///     Ok(_buf) => {}
///     Err(CompressionError::CompressionFailed) => eprintln!("compression failed"),
/// }
/// ```
#[derive(Error, Debug)]
pub enum CompressionError {
    /// raylib's compression helper returned a null or zero-length buffer.
    ///
    /// **Cause:** [`ffi::CompressData`](crate::ffi::CompressData) or
    /// [`ffi::DecompressData`](crate::ffi::DecompressData) failed internally — usually
    /// invalid or truncated input data, or an allocator failure.
    ///
    /// **Recovery:** Verify the input is non-empty and (for decompression) a complete
    /// DEFLATE payload produced by raylib's matching compress call.
    #[error("could not compress data")]
    CompressionFailed,
}

/// Errors returned when encoding or decoding Base64 data.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::Base64Error;
///
/// match raylib::core::data::decode_data_base64(b"AAAA") {
///     Ok(_buf) => {}
///     Err(Base64Error::DecodeFailed) => eprintln!("not valid base64"),
///     Err(Base64Error::EncodeFailed) => eprintln!("encode failed"),
/// }
/// ```
#[derive(Error, Debug)]
pub enum Base64Error {
    /// raylib's [`ffi::DecodeDataBase64`](crate::ffi::DecodeDataBase64) failed.
    ///
    /// **Cause:** The input contained characters outside the Base64 alphabet, was missing
    /// padding, or its length was inconsistent.
    ///
    /// **Recovery:** Re-validate the input is well-formed Base64 (matching `[A-Za-z0-9+/=]`)
    /// and re-attempt.
    #[error("could not decode base64 data")]
    DecodeFailed,
    /// raylib's [`ffi::EncodeDataBase64`](crate::ffi::EncodeDataBase64) failed.
    ///
    /// **Cause:** The encoder returned a null buffer, typically because the input length
    /// could not be expressed in the FFI's integer type or because the allocator failed.
    ///
    /// **Recovery:** Shrink the input, or pre-allocate sufficient memory; very large
    /// payloads should be encoded in chunks.
    #[error("could not encode base64 data")]
    EncodeFailed,
}

/// Errors returned when loading a `Model` resource.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::LoadModelError;
///
/// fn handle(e: LoadModelError) {
///     match e {
///         LoadModelError::LoadFromFileFailed { path } => eprintln!("model load failed: {path}"),
///         LoadModelError::LoadFromMeshFailed => eprintln!("model from mesh failed"),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum LoadModelError {
    /// raylib's [`ffi::LoadModel`](crate::ffi::LoadModel) returned a model with no meshes,
    /// materials, or skeleton.
    ///
    /// **Cause:** The file at `path` is missing, unreadable, or in a format raylib does
    /// not support; the resulting [`ffi::Model`](crate::ffi::Model) had all of its core
    /// pointers null.
    ///
    /// **Recovery:** Confirm the file exists and uses one of raylib's supported model
    /// formats (`.obj`, `.iqm`, `.gltf`, `.glb`, `.vox`, `.m3d`).
    #[error("could not load model\npath: {path:?}")]
    LoadFromFileFailed {
        /// Path to the model file raylib was unable to open.
        path: String,
    },
    /// raylib's [`ffi::LoadModelFromMesh`](crate::ffi::LoadModelFromMesh) returned a model
    /// with null mesh or material pointers.
    ///
    /// **Cause:** The supplied [`ffi::Mesh`](crate::ffi::Mesh) was invalid, or raylib's
    /// allocator failed while building the wrapping model.
    ///
    /// **Recovery:** Validate the mesh via [`InvalidMeshError`] before wrapping, and retry
    /// once the mesh passes validation.
    #[error("could not load model from mesh")]
    LoadFromMeshFailed,
}

/// Errors returned when loading model animations from a file.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::LoadModelAnimError;
///
/// fn handle(e: LoadModelAnimError) {
///     match e {
///         LoadModelAnimError::NoAnimationsLoaded { path } => {
///             eprintln!("no animations in {path}");
///         }
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum LoadModelAnimError {
    /// raylib's [`ffi::LoadModelAnimations`](crate::ffi::LoadModelAnimations) reported zero
    /// animations or a null array pointer.
    ///
    /// **Cause:** The file at `path` does not contain skeletal animations, or its format
    /// is not one raylib's animation loader understands (typically `.iqm`, `.gltf`, `.glb`,
    /// or `.m3d`).
    ///
    /// **Recovery:** Re-export the asset with an animation track baked in, or load the
    /// model without expecting animation data.
    #[error("no model animations loaded\npath: {path:?}")]
    NoAnimationsLoaded {
        /// Path to the model file that contained no animation tracks.
        path: String,
    },
}

/// Errors returned when assigning a material to a mesh slot on a model.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::SetMaterialError;
///
/// fn handle(e: SetMaterialError) {
///     match e {
///         SetMaterialError::MeshIdOutOfBounds => eprintln!("mesh_id past end of model"),
///         SetMaterialError::MaterialIdOutOfBounds => eprintln!("material_id past end of model"),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum SetMaterialError {
    /// The caller's `mesh_id` is `>= model.meshCount`.
    ///
    /// **Cause:** raylib-rs bounds-checks the mesh index against the [`ffi::Model`](crate::ffi::Model)'s
    /// `meshCount` field before forwarding to
    /// [`ffi::SetModelMeshMaterial`](crate::ffi::SetModelMeshMaterial), which would
    /// otherwise read garbage.
    ///
    /// **Recovery:** Use a `mesh_id` in `0..model.meshCount`.
    #[error("mesh_id greater than mesh count")]
    MeshIdOutOfBounds,
    /// The caller's `material_id` is `>= model.materialCount`.
    ///
    /// **Cause:** raylib-rs bounds-checks the material index against the [`ffi::Model`](crate::ffi::Model)'s
    /// `materialCount` field before forwarding to
    /// [`ffi::SetModelMeshMaterial`](crate::ffi::SetModelMeshMaterial).
    ///
    /// **Recovery:** Use a `material_id` in `0..model.materialCount`.
    #[error("material_id greater than material count")]
    MaterialIdOutOfBounds,
}

/// Errors returned when loading materials from a file.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::models::Material;
/// use raylib::core::error::LoadMaterialError;
///
/// match Material::load_materials("assets/scene.mtl") {
///     Ok(materials) => { /* use materials */ }
///     Err(LoadMaterialError::NoneLoaded { path }) => {
///         eprintln!("no materials found in {path}");
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum LoadMaterialError {
    /// raylib's [`ffi::LoadMaterials`](crate::ffi::LoadMaterials) returned zero materials.
    ///
    /// **Cause:** The file at `path` does not contain any material definitions raylib's
    /// material loader recognized — typically a malformed `.mtl` or a model file with no
    /// embedded materials.
    ///
    /// **Recovery:** Verify the `.mtl` (or model) file lists at least one material, or
    /// fall back to a default material constructed via [`ffi::LoadMaterialDefault`](crate::ffi::LoadMaterialDefault).
    #[error("no materials loaded\npath: {path:?}")]
    NoneLoaded {
        /// Path to the material source file that yielded no materials.
        path: String,
    },
}

/// Errors returned when loading a `Font` resource.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::LoadFontError;
///
/// fn handle(e: LoadFontError) {
///     match e {
///         LoadFontError::LoadFromFileFailed { path } => eprintln!("font file load failed: {path}"),
///         LoadFontError::LoadFromImageFailed => eprintln!("font from image failed"),
///         LoadFontError::LoadFromMemoryFailed => eprintln!("font from memory failed"),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum LoadFontError {
    /// raylib's [`ffi::LoadFont`](crate::ffi::LoadFont) returned a font whose glyph or
    /// texture data was empty.
    ///
    /// **Cause:** The font file at `path` was missing, unreadable, or in a format raylib
    /// does not support.
    ///
    /// **Recovery:** Confirm the file exists and uses one of `.ttf`, `.otf`, `.fnt`, or
    /// `.png` (for bitmap fonts).
    #[error(
        "error loading font; check if the file exists and if it's the right type\npath: {path:?}"
    )]
    LoadFromFileFailed {
        /// Path to the font file raylib was unable to open.
        path: String,
    },
    /// raylib's [`ffi::LoadFontFromImage`](crate::ffi::LoadFontFromImage) returned an empty
    /// font.
    ///
    /// **Cause:** The supplied image did not contain a recognizable glyph atlas — the key
    /// color was missing or the layout did not produce any cells.
    ///
    /// **Recovery:** Confirm the image follows raylib's bitmap-font atlas layout and that
    /// the chosen key color appears on the separators.
    #[error("error loading font from image")]
    LoadFromImageFailed,
    /// raylib's [`ffi::LoadFontFromMemory`](crate::ffi::LoadFontFromMemory) returned an
    /// empty font.
    ///
    /// **Cause:** The byte buffer was not a complete font file, or the supplied file-type
    /// hint (e.g. `".ttf"`) did not match the bytes.
    ///
    /// **Recovery:** Verify the buffer holds a complete font file and that the file-type
    /// extension hint matches its actual format.
    #[error("error loading font from memory; check if the file's type is correct")]
    LoadFromMemoryFailed,
}

/// Errors returned when an `Image` is invalid or cannot be processed.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::texture::Image;
/// use raylib::core::error::InvalidImageError;
///
/// match Image::load_image("assets/sprite.png") {
///     Ok(img) => { /* use img */ }
///     Err(InvalidImageError::NullDataFromFile) => {
///         eprintln!("file missing or unsupported format");
///     }
///     Err(InvalidImageError::InvalidFile) => {
///         eprintln!("file data malformed");
///     }
///     Err(_) => eprintln!("other image-load error"),
/// }
/// ```
#[derive(Error, Debug)]
pub enum InvalidImageError {
    /// The image's `width` field is zero.
    ///
    /// **Cause:** A zero-width image cannot be drawn, sampled, or uploaded as a texture;
    /// raylib-rs rejects it before any FFI call that would dereference its pixel buffer.
    ///
    /// **Recovery:** Re-generate or re-load the image with a positive width.
    #[error("invalid image: width is 0")]
    ZeroWidth,
    /// The image's `height` field is zero.
    ///
    /// **Cause:** A zero-height image cannot be drawn, sampled, or uploaded as a texture;
    /// raylib-rs rejects it before any FFI call that would dereference its pixel buffer.
    ///
    /// **Recovery:** Re-generate or re-load the image with a positive height.
    #[error("invalid image: height is 0")]
    ZeroHeight,
    /// The image's `data` pointer is null.
    ///
    /// **Cause:** The pixel buffer pointer is null even though width and height are
    /// non-zero — typically a partially-constructed image.
    ///
    /// **Recovery:** Re-load or re-generate the image from a known-valid source.
    #[error("invalid image: data is null")]
    NullData,
    /// raylib's [`ffi::LoadImage`](crate::ffi::LoadImage) returned an image with a null
    /// data pointer.
    ///
    /// **Cause:** The file does not exist, is unreadable, or is in a format raylib's image
    /// loaders do not support.
    ///
    /// **Recovery:** Confirm the file exists and uses one of `.png`, `.bmp`, `.tga`,
    /// `.jpg`, `.gif`, `.qoi`, `.psd`, `.dds`, `.hdr`, `.ktx`, `.astc`, `.pkm`, `.pvr`.
    #[error("image data is null, either the file doesnt exist or the image type is unsupported")]
    NullDataFromFile,
    /// raylib reported the file contents could not be parsed as an image.
    ///
    /// **Cause:** The file's header is missing, truncated, or otherwise malformed for its
    /// declared format.
    ///
    /// **Recovery:** Re-export the image from the source program, or open it in a viewer
    /// to confirm it is not corrupt.
    #[error("invalid file data")]
    InvalidFile,
    /// raylib's [`ffi::LoadImageFromMemory`](crate::ffi::LoadImageFromMemory) returned an
    /// image with a null data pointer.
    ///
    /// **Cause:** The supplied byte buffer was empty, truncated, or did not match the
    /// file-type extension hint.
    ///
    /// **Recovery:** Verify the buffer holds a complete image file and that the
    /// file-extension hint matches its actual format.
    #[error("image data is null, check provided buffer data")]
    NullDataFromMemory,
    /// raylib's [`ffi::LoadImageFromTexture`](crate::ffi::LoadImageFromTexture) returned an
    /// image with a null data pointer.
    ///
    /// **Cause:** The GPU readback from the source texture failed — usually an unsupported
    /// pixel format or a context-lost situation.
    ///
    /// **Recovery:** Confirm the source texture uses a CPU-readable
    /// [`ffi::PixelFormat`](crate::ffi::PixelFormat) (e.g. `PIXELFORMAT_UNCOMPRESSED_R8G8B8A8`)
    /// and that the GPU context is still current.
    #[error("failed to retrieve pixel data")]
    NullDataFromTexture,
    /// The image's pixel format is not supported by the requested operation.
    ///
    /// **Cause:** raylib's image processing helpers only accept a subset of pixel formats;
    /// the supplied image uses one outside that set (commonly the compressed formats).
    ///
    /// **Recovery:** Convert the image to a supported
    /// [`ffi::PixelFormat`](crate::ffi::PixelFormat) (typically `PIXELFORMAT_UNCOMPRESSED_R8G8B8A8`)
    /// via [`ffi::ImageFormat`](crate::ffi::ImageFormat) before re-attempting.
    #[error("unsupported format")]
    UnsupportedFormat,
    /// The convolution kernel's width and height differ.
    ///
    /// **Cause:** raylib's convolution helper requires a square kernel; raylib-rs
    /// bounds-checks `kernel.len()` against an integer square root before calling
    /// [`ffi::ImageKernelConvolution`](crate::ffi::ImageKernelConvolution).
    ///
    /// **Recovery:** Resize the kernel so its length is a perfect square (`9`, `16`,
    /// `25`, ...).
    #[error("convolution kernel must be square to be applied")]
    NonSquareKernel,
}

/// Errors returned when updating texture data on the GPU.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::UpdateTextureError;
///
/// fn handle(e: UpdateTextureError) {
///     match e {
///         UpdateTextureError::WrongDataSize { expect, actual } => {
///             eprintln!("texture expects {expect} bytes, got {actual}");
///         }
///         UpdateTextureError::OutOfBounds => eprintln!("region exceeds texture bounds"),
///         UpdateTextureError::NegativeSize => eprintln!("region has negative extents"),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum UpdateTextureError {
    /// The pixel-buffer slice does not match the byte size of the texture region.
    ///
    /// **Cause:** raylib-rs computed `width * height * bytes_per_pixel` for the target
    /// region and the supplied slice was a different length; passing it to
    /// [`ffi::UpdateTexture`](crate::ffi::UpdateTexture) would read out of bounds.
    ///
    /// **Recovery:** Size the input buffer to exactly `expect` bytes (region width times
    /// height times bytes per pixel for the texture's format).
    #[error("data is wrong size (expected {expect} bytes, got {actual})")]
    WrongDataSize {
        /// Byte count raylib-rs derived from the texture region's dimensions and format.
        expect: usize,
        /// Byte count of the slice the caller actually supplied.
        actual: usize,
    },
    /// The destination rectangle extends past the texture's `width` or `height`.
    ///
    /// **Cause:** The rectangle's `x + width` or `y + height` exceeds the texture
    /// dimensions; raylib-rs blocks this before [`ffi::UpdateTextureRec`](crate::ffi::UpdateTextureRec)
    /// would otherwise overflow.
    ///
    /// **Recovery:** Clamp the rectangle so it lies entirely within the texture's bounds.
    #[error("destination rectangle cannot exceed texture bounds")]
    OutOfBounds,
    /// The destination rectangle has a negative width or height.
    ///
    /// **Cause:** raylib treats the rectangle's `width`/`height` as unsigned at the FFI
    /// boundary; raylib-rs rejects negative values up front so they cannot wrap into very
    /// large positive sizes.
    ///
    /// **Recovery:** Pass non-negative extents; swap the corners if the caller's
    /// coordinates were inverted.
    #[error("destination rectangle cannot have negative extents")]
    NegativeSize,
}

/// Errors returned when loading a `Texture` resource.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::LoadTextureError;
///
/// fn handle(e: LoadTextureError) {
///     match e {
///         LoadTextureError::TextureFromFileFailed { path } => eprintln!("texture file load failed: {path}"),
///         LoadTextureError::CubemapFromImageFailed => eprintln!("cubemap from image failed"),
///         LoadTextureError::TextureFromImageFailed => eprintln!("texture from image failed"),
///         LoadTextureError::CreateRenderTextureFailed => eprintln!("render texture creation failed"),
///         LoadTextureError::InvalidData => eprintln!("texture data invalid"),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum LoadTextureError {
    /// raylib's [`ffi::LoadTexture`](crate::ffi::LoadTexture) returned a texture with id 0.
    ///
    /// **Cause:** The file at `path` was missing, unreadable, or in a format raylib's
    /// image loaders do not support; the resulting GPU texture id was the sentinel 0.
    ///
    /// **Recovery:** Confirm the file exists and uses a supported format; for compressed
    /// formats (`.dds`, `.ktx`, `.astc`), confirm the GPU supports the encoding.
    #[error("failed to load the texture\npath: {path:?}")]
    TextureFromFileFailed {
        /// Path to the image file raylib was unable to upload as a texture.
        path: String,
    },
    /// raylib's [`ffi::LoadTextureCubemap`](crate::ffi::LoadTextureCubemap) returned a
    /// cubemap with id 0.
    ///
    /// **Cause:** The source image did not match one of raylib's recognized cubemap
    /// layouts (line/cross/panorama), or its dimensions could not be divided into six
    /// square faces.
    ///
    /// **Recovery:** Re-export the image in a supported cubemap layout, or supply six
    /// individual face images.
    #[error("failed to load image as a texture cubemap")]
    CubemapFromImageFailed,
    /// raylib's [`ffi::LoadTextureFromImage`](crate::ffi::LoadTextureFromImage) returned a
    /// texture with id 0.
    ///
    /// **Cause:** The supplied [`ffi::Image`](crate::ffi::Image) was invalid (null data,
    /// zero extents) or its pixel format is not supported by the active GL backend.
    ///
    /// **Recovery:** Validate the image via [`InvalidImageError`] checks and, if needed,
    /// convert it to a GL-uploadable pixel format before re-attempting.
    #[error("failed to load image as a texture")]
    TextureFromImageFailed,
    /// raylib's [`ffi::LoadRenderTexture`](crate::ffi::LoadRenderTexture) returned a
    /// render texture whose backing framebuffer id is 0.
    ///
    /// **Cause:** The GL backend rejected the framebuffer attachment — typically because
    /// the requested dimensions exceed `GL_MAX_RENDERBUFFER_SIZE` or the active context
    /// is lost.
    ///
    /// **Recovery:** Lower the framebuffer dimensions, or verify the GL context is current
    /// and not exhausted.
    #[error("failed to create render texture")]
    CreateRenderTextureFailed,
    /// The supplied data is not a recognized texture payload.
    ///
    /// **Cause:** raylib-rs validated the input (typically a raw pixel buffer) and found
    /// it inconsistent with the requested width/height/format before invoking the FFI.
    ///
    /// **Recovery:** Ensure the buffer length matches `width * height * bytes_per_pixel`
    /// for the requested pixel format.
    #[error("data is not valid to load texture")]
    InvalidData,
}

/// Top-level error type that aggregates all raylib-rs domain errors.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::RaylibError;
///
/// fn handle(e: RaylibError) {
///     match e {
///         RaylibError::AudioInit(_) => eprintln!("audio device failed to initialize"),
///         RaylibError::ExportWave(_) => eprintln!("wave export failed"),
///         RaylibError::LoadSound(_) => eprintln!("sound load failed"),
///         RaylibError::Allocation(_) => eprintln!("raylib allocator failed"),
///         RaylibError::Compression(_) => eprintln!("data compression failed"),
///         RaylibError::LoadModel(_) => eprintln!("model load failed"),
///         RaylibError::LoadModelAnim(_) => eprintln!("model animation load failed"),
///         RaylibError::SetMaterial(_) => eprintln!("material assignment out of bounds"),
///         RaylibError::LoadMaterial(_) => eprintln!("material load failed"),
///         RaylibError::LoadFont(_) => eprintln!("font load failed"),
///         RaylibError::InvalidImage(_) => eprintln!("image was invalid"),
///         RaylibError::UpdateTexture(_) => eprintln!("texture update failed"),
///         RaylibError::LoadTexture(_) => eprintln!("texture load failed"),
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum RaylibError {
    /// Wraps an [`AudioInitError`] surfaced through `?` from an audio init call site.
    ///
    /// **Cause:** A call to `RaylibAudio::init_audio_device()` (or another audio-init
    /// entry point) propagated up an [`AudioInitError`].
    ///
    /// **Recovery:** Inspect the inner [`AudioInitError`] for the specific reason
    /// (`DoubleInit` vs `InitFailed`) and apply that variant's recovery.
    #[error("audio initialization error")]
    AudioInit(#[from] AudioInitError),
    /// Wraps an [`ExportWaveError`] surfaced through `?` from a wave-export call site.
    ///
    /// **Cause:** A wave-export call (typically `Wave::export`) propagated up an
    /// [`ExportWaveError`].
    ///
    /// **Recovery:** Inspect the inner [`ExportWaveError`] for the specific reason and
    /// apply that variant's recovery.
    #[error("wave export error")]
    ExportWave(#[from] ExportWaveError),
    /// Wraps a [`LoadSoundError`] surfaced through `?` from a sound or music load site.
    ///
    /// **Cause:** A sound or music load call (e.g. `RaylibAudio::new_sound`,
    /// `RaylibAudio::new_music`) propagated up a [`LoadSoundError`].
    ///
    /// **Recovery:** Inspect the inner [`LoadSoundError`] variant and apply its recovery.
    #[error("sound loading error")]
    LoadSound(#[from] LoadSoundError),
    /// Wraps an [`AllocationError`] surfaced through `?` from any raylib-allocator call site.
    ///
    /// **Cause:** A path that calls [`ffi::MemAlloc`](crate::ffi::MemAlloc) (mesh upload,
    /// data buffer construction, etc.) propagated up an [`AllocationError`].
    ///
    /// **Recovery:** Inspect the inner [`AllocationError`] variant and apply its recovery
    /// (usually: reduce the request size).
    #[error("allocation error")]
    Allocation(#[from] AllocationError),
    /// Wraps a [`CompressionError`] surfaced through `?` from a data-compression call site.
    ///
    /// **Cause:** A `compress_data` or `decompress_data` call propagated up a
    /// [`CompressionError`].
    ///
    /// **Recovery:** Inspect the inner [`CompressionError`] variant and apply its recovery.
    #[error("compression error")]
    Compression(#[from] CompressionError),
    /// Wraps a [`LoadModelError`] surfaced through `?` from a model-load call site.
    ///
    /// **Cause:** A `load_model` or `load_model_from_mesh` call propagated up a
    /// [`LoadModelError`].
    ///
    /// **Recovery:** Inspect the inner [`LoadModelError`] variant and apply its recovery.
    #[error("model loading error")]
    LoadModel(#[from] LoadModelError),
    /// Wraps a [`LoadModelAnimError`] surfaced through `?` from a model-animation load site.
    ///
    /// **Cause:** A `load_model_animations` call propagated up a [`LoadModelAnimError`].
    ///
    /// **Recovery:** Inspect the inner [`LoadModelAnimError`] variant and apply its
    /// recovery.
    #[error("model animation loading error")]
    LoadModelAnim(#[from] LoadModelAnimError),
    /// Wraps a [`SetMaterialError`] surfaced through `?` from a material-assignment site.
    ///
    /// **Cause:** A `set_model_mesh_material` call propagated up a [`SetMaterialError`].
    ///
    /// **Recovery:** Inspect the inner [`SetMaterialError`] variant and apply its recovery
    /// (usually: clamp the mesh or material id).
    #[error("material update error")]
    SetMaterial(#[from] SetMaterialError),
    /// Wraps a [`LoadMaterialError`] surfaced through `?` from a material-load call site.
    ///
    /// **Cause:** A `load_materials` call propagated up a [`LoadMaterialError`].
    ///
    /// **Recovery:** Inspect the inner [`LoadMaterialError`] variant and apply its
    /// recovery.
    #[error("material loading error")]
    LoadMaterial(#[from] LoadMaterialError),
    /// Wraps a [`LoadFontError`] surfaced through `?` from a font-load call site.
    ///
    /// **Cause:** A `load_font`, `load_font_from_image`, or `load_font_from_memory` call
    /// propagated up a [`LoadFontError`].
    ///
    /// **Recovery:** Inspect the inner [`LoadFontError`] variant and apply its recovery.
    #[error("font loading error")]
    LoadFont(#[from] LoadFontError),
    /// Wraps an [`InvalidImageError`] surfaced through `?` from an image-processing site.
    ///
    /// **Cause:** An image load, conversion, or processing call propagated up an
    /// [`InvalidImageError`].
    ///
    /// **Recovery:** Inspect the inner [`InvalidImageError`] variant and apply its
    /// recovery.
    #[error("image error")]
    InvalidImage(#[from] InvalidImageError),
    /// Wraps an [`UpdateTextureError`] surfaced through `?` from a texture-update site.
    ///
    /// **Cause:** An `update_texture` or `update_texture_rec` call propagated up an
    /// [`UpdateTextureError`].
    ///
    /// **Recovery:** Inspect the inner [`UpdateTextureError`] variant and apply its
    /// recovery.
    #[error("texture update error")]
    UpdateTexture(#[from] UpdateTextureError),
    /// Wraps a [`LoadTextureError`] surfaced through `?` from a texture-load call site.
    ///
    /// **Cause:** A `load_texture`, `load_texture_from_image`, `load_texture_cubemap`, or
    /// `load_render_texture` call propagated up a [`LoadTextureError`].
    ///
    /// **Recovery:** Inspect the inner [`LoadTextureError`] variant and apply its recovery.
    #[error("texture loading error")]
    LoadTexture(#[from] LoadTextureError),
}
