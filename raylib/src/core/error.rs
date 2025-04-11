//! Definitions for error types used throught the crate

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioInitError {
    #[error("RaylibAudio cannot be instantiated more then once at a time")]
    DoubleInit,
}

#[derive(Error, Debug)]
pub enum LoadSoundError {
    #[error("failed to load sound\npath: {path:?}")]
    LoadFailed { path: String },
    #[error("failed to load sound from wave")]
    LoadFromWaveFailed,
    #[error("cannot load wave\npath: {path:?}")]
    LoadWaveFromFileFailed { path: String },
    #[error("wave data is null, check provided buffer data")]
    Null,
    #[error("music could not be loaded from file\npath: {path:?}")]
    LoadMusicFromFileFailed { path: String },
    #[error("music's buffer data data is null, check provided buffer data")]
    MusicNull,
}

#[derive(Error, Debug)]
pub enum AllocationError {
    #[error("memory request does not produce a valid layout")]
    InvalidLayout,
    #[error("memory request exceeds capacity")]
    ExceedsCapacity,
    #[error("memory request exceeds unsigned integer maximum")]
    ExceedsUIntMax,
    #[error("cannot allocate less than 1 element")]
    SubMinSize,
}

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("could not compress data")]
    CompressionFailed,
}

#[derive(Error, Debug)]
pub enum LoadModelError {
    #[error("could not load model\npath: {path:?}")]
    LoadFromFileFailed { path: String },
    #[error("could not load model from mesh")]
    LoadFromMeshFailed,
}

#[derive(Error, Debug)]
pub enum LoadModelAnimError {
    #[error("no model animations loaded\npath: {path:?}")]
    NoAnimationsLoaded { path: String },
}

#[derive(Error, Debug)]
pub enum SetMaterialError {
    #[error("mesh_id greater than mesh count")]
    MeshIdOutOfBounds,
    #[error("material_id greater than material count")]
    MaterialIdOutOfBounds,
}

#[derive(Error, Debug)]
pub enum LoadMaterialError {
    #[error("no materials loaded\npath: {path:?}")]
    NoneLoaded { path: String },
}

#[derive(Error, Debug)]
pub enum LoadFontError {
    #[error("error loading font; check if the file exists and if it's the right type\npath: {path:?}")]
    LoadFromFileFailed { path: String },
    #[error("error loading font from image")]
    LoadFromImageFailed,
    #[error("error loading font from memory; check if the file's type is correct")]
    LoadFromMemoryFailed,
}

#[derive(Error, Debug)]
pub enum InvalidImageError {
    #[error("invalid image: width is 0")]
    ZeroWidth,
    #[error("invalid image: height is 0")]
    ZeroHeight,
    #[error("invalid image: data is null")]
    NullData,
    #[error("image data is null, either the file doesnt exist or the image type is unsupported")]
    NullDataFromFile,
    #[error("image data is null, check provided buffer data")]
    NullDataFromMemory,
    #[error("texture could not be rendered to an image")]
    NullDataFromTexture,
    #[error("unsupported format")]
    UnsupportedFormat,
    #[error("convolution kernel must be square to be applied")]
    NonSquareKernel,
}

#[derive(Error, Debug)]
pub enum UpdateTextureError {
    #[error("data is wrong size (expected {expect} bytes, got {actual})")]
    WrongDataSize { expect: usize, actual: usize },
    #[error("destination rectangle cannot exceed texture bounds")]
    OutOfBounds,
    #[error("destination rectangle cannot have negative extents")]
    NegativeSize,
}

#[derive(Error, Debug)]
pub enum LoadTextureError {
    #[error("failed to load the texture\npath: {path:?}")]
    TextureFromFileFailed { path: String },
    #[error("failed to load image as a texture cubemap")]
    CubemapFromImageFailed,
    #[error("failed to load image as a texture")]
    TextureFromImageFailed,
    #[error("failed to create render texture")]
    CreateRenderTextureFailed,
}

#[derive(Error, Debug)]
pub enum RaylibError {
    #[error("audio initialization error")]
    AudioInitError(#[from] AudioInitError),
    #[error("sound loading error")]
    LoadSoundError(#[from] LoadSoundError),
    #[error("allocation error")]
    AllocationError(#[from] AllocationError),
    #[error("compression error")]
    CompressionError(#[from] CompressionError),
    #[error("model loading error")]
    LoadModelError(#[from] LoadModelError),
    #[error("model animation loading error")]
    LoadModelAnimError(#[from] LoadModelAnimError),
    #[error("material update error")]
    SetMaterialError(#[from] SetMaterialError),
    #[error("material loading error")]
    LoadMaterialError(#[from] LoadMaterialError),
    #[error("font loading error")]
    LoadFontError(#[from] LoadFontError),
    #[error("image error")]
    InvalidImageError(#[from] InvalidImageError),
    #[error("texture update error")]
    UpdateTextureError(#[from] UpdateTextureError),
    #[error("texture loading error")]
    LoadTextureError(#[from] LoadTextureError),
}
