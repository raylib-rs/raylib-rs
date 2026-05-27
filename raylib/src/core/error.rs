//! Definitions for error types used throughout the crate

use thiserror::Error;

/// Errors returned when initializing the raylib audio device.
#[derive(Error, Debug)]
pub enum AudioInitError {
    /// Audio was already initialized; only one audio device may be active at a time.
    #[error("RaylibAudio cannot be instantiated more then once at a time")]
    DoubleInit,
    /// The underlying audio device failed to initialize.
    #[error("failed to initialize audio device")]
    InitFailed,
}

/// Errors returned when exporting wave data to a file.
#[derive(Error, Debug)]
pub enum ExportWaveError {
    /// The wave data must be 16-bit per sample for QOA format export, but a different depth was provided.
    #[error("wave data must be 16 bit per sample for QOA format export (actual: {0})")]
    QoaBadSamples(i32),
    /// The wave export operation failed.
    #[error("failed to export wave data")]
    ExportFailed,
}

/// Errors returned when loading a `Sound` or `Music` resource.
#[derive(Error, Debug)]
pub enum LoadSoundError {
    /// Failed to load a sound from a file at the given path.
    #[error("failed to load sound\npath: {path:?}")]
    LoadFailed {
        /// The file path that failed to load.
        path: String,
    },
    /// Failed to load a sound from a `Wave` object.
    #[error("failed to load sound from wave")]
    LoadFromWaveFailed,
    /// Failed to load wave data from the given file path.
    #[error("cannot load wave\npath: {path:?}")]
    LoadWaveFromFileFailed {
        /// The file path that failed to load.
        path: String,
    },
    /// The wave data pointer returned by raylib was null.
    #[error("wave data is null, check provided buffer data")]
    Null,
    /// Failed to load music from the given file path.
    #[error("music could not be loaded from file\npath: {path:?}")]
    LoadMusicFromFileFailed {
        /// The file path that failed to load.
        path: String,
    },
    /// The music buffer data pointer returned by raylib was null.
    #[error("music's buffer data data is null, check provided buffer data")]
    MusicNull,
}

/// Errors that can occur when pushing new audio data into a `Sound` or `AudioStream`.
/// **Notes** (iann): if raylib upstream discussion introduces any of these checks, we might simplify these to avoid any redundancy i think
/// 1. `SampleSizeMismatch` is raylib-rs only, raylib does not do sampleSize matching checks.
/// 2. `TooManyFrames` comes from the WARNING behavior in raylib `UpdateAudioStreamInLockedState`: <https://github.com/raysan5/raylib/blob/master/src/raudio.c#L2662>
#[derive(Error, Debug)]
pub enum UpdateAudioStreamError {
    /// The audio data's sample bit depth does not match what the stream expects.
    #[error("update data format must match sound: expected {expected} bits, got {provided} bits")]
    SampleSizeMismatch {
        /// The sample bit depth expected by the audio stream.
        expected: usize,
        /// The sample bit depth of the provided data.
        provided: usize,
    },
    /// The number of frames provided exceeds the audio buffer's remaining capacity.
    #[error("Attempting to write too many frames to buffer: provided {provided}, max {max}")]
    TooManyFrames {
        /// The maximum number of frames the buffer can accept.
        max: usize,
        /// The number of frames that were provided.
        provided: usize,
    },
    /// The audio stream's callback slot is already occupied by another callback.
    #[error(
        "AudioStream's callback slot is already in use; call unset_audio_stream_callback() to clear it"
    )]
    CallbackSlotBusy,
}

/// Errors returned when a memory allocation via raylib's allocator fails.
#[derive(Error, Debug)]
pub enum AllocationError {
    /// [`MemAlloc`](crate::ffi::MemAlloc) returned null.
    #[error("memory request exceeds capacity")]
    NullAlloc,
    /// The size of `[T; count]` in bytes exceeds [`u32::MAX`]
    /// (the largest value [`MemAlloc`](crate::ffi::MemAlloc) can be passed).
    #[error("memory request in bytes exceeds unsigned integer maximum")]
    IntoUIntFailed,
    /// Attempted to pass 0 to [`MemAlloc`](crate::ffi::MemAlloc).
    #[error("requested zero bytes of memory")]
    ZeroBytes,
}

/// Errors returned when validating a `Mesh` before upload or generation.
#[derive(Error, Debug)]
pub enum InvalidMeshError {
    /// The mesh vertex or index count is not a multiple of three, as required for triangle lists.
    #[error("mesh should have 3 indices/vertices for each triangle")]
    TrianglePointMiscount,
    /// One or more indices reference a vertex position beyond the vertex buffer.
    #[error("indices should be within the number of vertices")]
    IndexOutOfBounds,
    /// The vertex count exceeds `u16::MAX`, which is the maximum for indexed meshes.
    #[error("mesh with indices should not exceed u16::MAX vertices")]
    VertexUnindexible(std::num::TryFromIntError),
    /// The number of primary UV coordinates does not match the vertex count.
    #[error("mesh should have one texcoord per vertex")]
    TexcoordsMiscount,
    /// The number of secondary UV coordinates does not match the vertex count.
    #[error("mesh with texcoords2 should have one per vertex")]
    Texcoords2Miscount,
    /// The number of vertex normals does not match the vertex count.
    #[error("mesh with normals should have one per vertex")]
    NormalsMiscount,
    /// The number of vertex tangents does not match the vertex count.
    #[error("mesh with tangents should have one per vertex")]
    TangentsMiscount,
    /// The number of vertex colors does not match the vertex count.
    #[error("mesh with colors should have one per vertex")]
    ColorsMiscount,
}

/// Errors returned when procedurally generating a mesh.
#[derive(Error, Debug)]
pub enum GenMeshError {
    /// The provided mesh data failed validation.
    #[error("provided mesh data does not correspond to a valid mesh")]
    InvalidMesh(#[from] InvalidMeshError),
    /// A memory allocation for the mesh data failed.
    #[error("could not allocate memory for the mesh data")]
    Allocation(#[from] AllocationError),
}

/// Errors returned when compressing data.
#[derive(Error, Debug)]
pub enum CompressionError {
    /// The compression operation failed.
    #[error("could not compress data")]
    CompressionFailed,
}

/// Errors returned when encoding or decoding Base64 data.
#[derive(Error, Debug)]
pub enum Base64Error {
    /// The Base64 decoding operation failed.
    #[error("could not decode base64 data")]
    DecodeFailed,
    /// The Base64 encoding operation failed.
    #[error("could not encode base64 data")]
    EncodeFailed,
}

/// Errors returned when loading a `Model` resource.
#[derive(Error, Debug)]
pub enum LoadModelError {
    /// Failed to load a model from the given file path.
    #[error("could not load model\npath: {path:?}")]
    LoadFromFileFailed {
        /// The file path that failed to load.
        path: String,
    },
    /// Failed to load a model from a `Mesh` object.
    #[error("could not load model from mesh")]
    LoadFromMeshFailed,
}

/// Errors returned when loading model animations from a file.
#[derive(Error, Debug)]
pub enum LoadModelAnimError {
    /// No animations were found in the file at the given path.
    #[error("no model animations loaded\npath: {path:?}")]
    NoAnimationsLoaded {
        /// The file path that contained no animations.
        path: String,
    },
}

/// Errors returned when assigning a material to a mesh slot on a model.
#[derive(Error, Debug)]
pub enum SetMaterialError {
    /// The provided mesh index is out of range for the model's mesh list.
    #[error("mesh_id greater than mesh count")]
    MeshIdOutOfBounds,
    /// The provided material index is out of range for the model's material list.
    #[error("material_id greater than material count")]
    MaterialIdOutOfBounds,
}

/// Errors returned when loading materials from a file.
#[derive(Error, Debug)]
pub enum LoadMaterialError {
    /// No materials were found in the file at the given path.
    #[error("no materials loaded\npath: {path:?}")]
    NoneLoaded {
        /// The file path that contained no materials.
        path: String,
    },
}

/// Errors returned when loading a `Font` resource.
#[derive(Error, Debug)]
pub enum LoadFontError {
    /// Failed to load a font from the given file path.
    #[error(
        "error loading font; check if the file exists and if it's the right type\npath: {path:?}"
    )]
    LoadFromFileFailed {
        /// The file path that failed to load.
        path: String,
    },
    /// Failed to load a font from an `Image`.
    #[error("error loading font from image")]
    LoadFromImageFailed,
    /// Failed to load a font from a memory buffer.
    #[error("error loading font from memory; check if the file's type is correct")]
    LoadFromMemoryFailed,
}

/// Errors returned when an `Image` is invalid or cannot be processed.
#[derive(Error, Debug)]
pub enum InvalidImageError {
    /// The image has a width of zero.
    #[error("invalid image: width is 0")]
    ZeroWidth,
    /// The image has a height of zero.
    #[error("invalid image: height is 0")]
    ZeroHeight,
    /// The image pixel data pointer is null.
    #[error("invalid image: data is null")]
    NullData,
    /// The image pixel data was null after loading from file, indicating the file does not exist or its format is unsupported.
    #[error("image data is null, either the file doesnt exist or the image type is unsupported")]
    NullDataFromFile,
    /// The image file data is invalid or corrupted.
    #[error("invalid file data")]
    InvalidFile,
    /// The image pixel data was null after loading from a memory buffer.
    #[error("image data is null, check provided buffer data")]
    NullDataFromMemory,
    /// Failed to read pixel data back from a GPU texture.
    #[error("failed to retrieve pixel data")]
    NullDataFromTexture,
    /// The image pixel format is not supported for this operation.
    #[error("unsupported format")]
    UnsupportedFormat,
    /// The convolution kernel is not square (width != height).
    #[error("convolution kernel must be square to be applied")]
    NonSquareKernel,
}

/// Errors returned when updating texture data on the GPU.
#[derive(Error, Debug)]
pub enum UpdateTextureError {
    /// The byte size of the provided data does not match the expected texture region size.
    #[error("data is wrong size (expected {expect} bytes, got {actual})")]
    WrongDataSize {
        /// The number of bytes expected for the texture region.
        expect: usize,
        /// The number of bytes actually provided.
        actual: usize,
    },
    /// The destination rectangle extends outside the texture bounds.
    #[error("destination rectangle cannot exceed texture bounds")]
    OutOfBounds,
    /// The destination rectangle has negative width or height.
    #[error("destination rectangle cannot have negative extents")]
    NegativeSize,
}

/// Errors returned when loading a `Texture` resource.
#[derive(Error, Debug)]
pub enum LoadTextureError {
    /// Failed to load a texture from the given file path.
    #[error("failed to load the texture\npath: {path:?}")]
    TextureFromFileFailed {
        /// The file path that failed to load.
        path: String,
    },
    /// Failed to load an image as a cubemap texture.
    #[error("failed to load image as a texture cubemap")]
    CubemapFromImageFailed,
    /// Failed to load an image as a 2D texture.
    #[error("failed to load image as a texture")]
    TextureFromImageFailed,
    /// Failed to create a render texture (framebuffer).
    #[error("failed to create render texture")]
    CreateRenderTextureFailed,
    /// The provided data is not valid for texture loading.
    #[error("data is not valid to load texture")]
    InvalidData,
}

/// Top-level error type that aggregates all raylib-rs domain errors.
#[derive(Error, Debug)]
pub enum RaylibError {
    /// An error occurred during audio initialization.
    #[error("audio initialization error")]
    AudioInit(#[from] AudioInitError),
    /// An error occurred while exporting wave data.
    #[error("wave export error")]
    ExportWave(#[from] ExportWaveError),
    /// An error occurred while loading a sound or music resource.
    #[error("sound loading error")]
    LoadSound(#[from] LoadSoundError),
    /// A memory allocation via raylib's allocator failed.
    #[error("allocation error")]
    Allocation(#[from] AllocationError),
    /// A data compression operation failed.
    #[error("compression error")]
    Compression(#[from] CompressionError),
    /// An error occurred while loading a model.
    #[error("model loading error")]
    LoadModel(#[from] LoadModelError),
    /// An error occurred while loading model animations.
    #[error("model animation loading error")]
    LoadModelAnim(#[from] LoadModelAnimError),
    /// An error occurred while assigning a material to a model mesh slot.
    #[error("material update error")]
    SetMaterial(#[from] SetMaterialError),
    /// An error occurred while loading materials from a file.
    #[error("material loading error")]
    LoadMaterial(#[from] LoadMaterialError),
    /// An error occurred while loading a font.
    #[error("font loading error")]
    LoadFont(#[from] LoadFontError),
    /// An image was invalid or could not be processed.
    #[error("image error")]
    InvalidImage(#[from] InvalidImageError),
    /// An error occurred while updating texture data on the GPU.
    #[error("texture update error")]
    UpdateTexture(#[from] UpdateTextureError),
    /// An error occurred while loading a texture.
    #[error("texture loading error")]
    LoadTexture(#[from] LoadTextureError),
}
