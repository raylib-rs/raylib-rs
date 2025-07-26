//! Definitions for error types used throughout the crate

use thiserror::Error;

#[allow(unused_imports, reason = "used in documentation")]
use crate::{
    audio::{Music, RaylibAudio, Sound, Wave},
    models::{Material, Mesh, Model, ModelAnimation},
    text::Font,
    texture::{Image, RenderTexture2D, Texture2D},
};

/// Error occurring while initializing audio.
#[derive(Error, Debug)]
pub enum AudioInitError {
    /// [`RaylibAudio`] cannot be instantiated more then once at a time
    #[error("RaylibAudio cannot be instantiated more then once at a time")]
    DoubleInit,
    /// Raylib failed to initialize audio device
    #[error("failed to initialize audio device")]
    InitFailed,
}

/// Error occurring while exporting a [`Wave`].
#[derive(Error, Debug)]
pub enum ExportWaveError {
    /// [`Wave`] data must be 16 bit per sample for QOA format export
    #[error("wave data must be 16 bit per sample for QOA format export (actual: {0})")]
    QoaBadSamples(i32),
    /// Failed to export [`Wave`] data
    #[error("failed to export wave data")]
    ExportFailed,
}

/// Error occurring while loading a [`Sound`].
#[derive(Error, Debug)]
pub enum LoadSoundError {
    /// Failed to load [`Sound`]
    #[error("failed to load sound\npath: {path:?}")]
    LoadFailed {
        /// Path to the file that failed to be loaded as a [`Sound`].
        path: String,
    },
    /// failed to load [`Sound`] from [`Wave`]
    #[error("failed to load sound from wave")]
    LoadFromWaveFailed,
    /// cannot load [`Wave`]
    #[error("cannot load wave\npath: {path:?}")]
    LoadWaveFromFileFailed {
        /// Path to the file that failed to be loaded as a [`Wave`].
        path: String,
    },
    /// [`Wave`] `data` is null
    #[error("wave data is null, check provided buffer data")]
    Null,
    /// [`Music`] could not be loaded from file
    #[error("music could not be loaded from file\npath: {path:?}")]
    LoadMusicFromFileFailed {
        /// Path to the file that failed to be loaded as [`Music`].
        path: String,
    },
    /// [`Music`] buffer `data` is null
    #[error("music's buffer data is null, check provided buffer data")]
    MusicNull,
}

/// Error occurring while allocating memory with [`MemAlloc`](crate::ffi::MemAlloc)/[`MemRealloc`](crate::ffi::MemRealloc).
#[derive(Error, Debug)]
pub enum AllocationError {
    /// [`MemAlloc`](crate::ffi::MemAlloc)/[`MemRealloc`](crate::ffi::MemRealloc) returned null.
    #[error("memory request exceeds capacity")]
    NullAlloc,
    /// The size of `[T; count]` in bytes exceeds [`u32::MAX`]
    /// (the largest value [`MemAlloc`](crate::ffi::MemAlloc)/[`MemRealloc`](crate::ffi::MemRealloc) can be passed).
    #[error("memory request in bytes exceeds unsigned integer maximum")]
    IntoUIntFailed,
    /// Attempted to pass 0 to [`MemAlloc`](crate::ffi::MemAlloc)/[`MemRealloc`](crate::ffi::MemRealloc).
    #[error("requested zero bytes of memory")]
    ZeroBytes,
}

/// Error occurring while compressing data.
#[derive(Error, Debug)]
pub enum CompressionError {
    /// Could not compress data
    #[error("could not compress data")]
    CompressionFailed,
}

/// Error occurring while loading a [`Model`].
#[derive(Error, Debug)]
pub enum LoadModelError {
    /// Could not load [`Model`]
    #[error("could not load model\npath: {path:?}")]
    LoadFromFileFailed {
        /// Path to the file that failed to be loaded as a [`Model`]
        path: String,
    },
    /// could not load [`Model`] from [`Mesh`]
    #[error("could not load model from mesh")]
    LoadFromMeshFailed,
}

/// Error occurring while loading a [`ModelAnimation`].
#[derive(Error, Debug)]
pub enum LoadModelAnimError {
    /// No [`ModelAnimation`]s loaded
    #[error("no model animations loaded\npath: {path:?}")]
    NoAnimationsLoaded {
        /// Path to the file that failed to be loaded as a [`ModelAnimation`].
        path: String,
    },
}

/// Error occurring while setting a [`Model`]'s [`Material`].
#[derive(Error, Debug)]
pub enum SetMaterialError {
    /// `mesh_id` greater than [`Mesh`] count
    #[error("mesh_id greater than mesh count")]
    MeshIdOutOfBounds,
    /// `material_id` greater than [`Material`] count
    #[error("material_id greater than material count")]
    MaterialIdOutOfBounds,
}

/// Error occurring while loading a [`Material`].
#[derive(Error, Debug)]
pub enum LoadMaterialError {
    /// No [`Material`]s loaded
    #[error("no materials loaded\npath: {path:?}")]
    NoneLoaded {
        /// Path to the file that failed to be loaded as a [`Material`].
        path: String,
    },
}

/// Error occurring while loading a [`Material`].
#[derive(Error, Debug)]
pub enum LoadFontError {
    /// Error loading [`Font`] from a file
    #[error(
        "error loading font; check if the file exists and if it's the right type\npath: {path:?}"
    )]
    LoadFromFileFailed {
        /// Path to the file that failed to be loaded as a [`Font`].
        path: String,
    },
    /// Error loading [`Font`] from [`Image`]
    #[error("error loading font from image")]
    LoadFromImageFailed,
    /// Error loading [`Font`] from memory
    #[error("error loading font from memory; check if the file's type is correct")]
    LoadFromMemoryFailed,
}

/// Error regarding [`Image`] data.
#[derive(Error, Debug)]
pub enum InvalidImageError {
    /// `width` is 0
    #[error("invalid image: width is 0")]
    ZeroWidth,
    /// `height` is 0
    #[error("invalid image: height is 0")]
    ZeroHeight,
    /// `data` is null
    #[error("invalid image: data is null")]
    NullData,
    /// `data` loaded from file is null
    #[error("image data is null, either the file doesnt exist or the image type is unsupported")]
    NullDataFromFile,
    /// Invalid file data
    #[error("invalid file data")]
    InvalidFile,
    /// `data` loaded from memory is null
    #[error("image data is null, check provided buffer data")]
    NullDataFromMemory,
    /// Failed to retrieve pixel data
    #[error("failed to retrieve pixel data")]
    NullDataFromTexture,
    /// Unsupported format
    #[error("unsupported format")]
    UnsupportedFormat,
    /// Convolution kernel must be square to be applied
    #[error("convolution kernel must be square to be applied")]
    NonSquareKernel,
}

/// Error occurring while updating a [`Texture2D`].
#[derive(Error, Debug)]
pub enum UpdateTextureError {
    /// Data is wrong size
    #[error("data is wrong size (expected {expect} bytes, got {actual})")]
    WrongDataSize {
        /// Bytes expected based on destination
        expect: usize,
        /// Bytes found based on source
        actual: usize,
    },
    /// Destination rectangle cannot exceed texture bounds
    #[error("destination rectangle cannot exceed texture bounds")]
    OutOfBounds,
    /// Destination rectangle cannot have negative extents
    #[error("destination rectangle cannot have negative extents")]
    NegativeSize,
}

/// Error occurring while loading a texture.
#[derive(Error, Debug)]
pub enum LoadTextureError {
    /// Failed to load the [`Texture2D`]
    #[error("failed to load the texture\npath: {path:?}")]
    TextureFromFileFailed {
        /// Path to the file that failed to be loaded as a [`Texture2D`].
        path: String,
    },
    /// Failed to load [`Image`] as a texture cubemap
    #[error("failed to load image as a texture cubemap")]
    CubemapFromImageFailed,
    /// Failed to load [`Image`] as a [`Texture2D`]
    #[error("failed to load image as a texture")]
    TextureFromImageFailed,
    /// Failed to create [`RenderTexture2D`]
    #[error("failed to create render texture")]
    CreateRenderTextureFailed,
    /// Data is not valid to load texture
    #[error("data is not valid to load texture")]
    InvalidData,
}

/// General Raylib error
#[derive(Error, Debug)]
pub enum RaylibError {
    /// [`AudioInitError`]
    #[error("audio initialization error")]
    AudioInit(#[from] AudioInitError),
    /// [`ExportWaveError`]
    #[error("wave export error")]
    ExportWave(#[from] ExportWaveError),
    /// [`LoadSoundError`]
    #[error("sound loading error")]
    LoadSound(#[from] LoadSoundError),
    /// [`AllocationError`]
    #[error("allocation error")]
    Allocation(#[from] AllocationError),
    /// [`CompressionError`]
    #[error("compression error")]
    Compression(#[from] CompressionError),
    /// [`LoadModelError`]
    #[error("model loading error")]
    LoadModel(#[from] LoadModelError),
    /// [`LoadModelAnimError`]
    #[error("model animation loading error")]
    LoadModelAnim(#[from] LoadModelAnimError),
    /// [`SetMaterialError`]
    #[error("material update error")]
    SetMaterial(#[from] SetMaterialError),
    /// [`LoadMaterialError`]
    #[error("material loading error")]
    LoadMaterial(#[from] LoadMaterialError),
    /// [`LoadFontError`]
    #[error("font loading error")]
    LoadFont(#[from] LoadFontError),
    /// [`InvalidImageError`]
    #[error("image error")]
    InvalidImage(#[from] InvalidImageError),
    /// [`UpdateTextureError`]
    #[error("texture update error")]
    UpdateTexture(#[from] UpdateTextureError),
    /// [`LoadTextureError`]
    #[error("texture loading error")]
    LoadTexture(#[from] LoadTextureError),
}
