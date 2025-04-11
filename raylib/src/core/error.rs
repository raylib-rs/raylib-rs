//! Definitions for error types used throught the crate

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RaylibAudioInitError {
    #[error("RaylibAudio cannot be instantiated more then once at a time")]
    DoubleInit,
}

#[derive(Error, Debug)]
pub enum RaylibLoadSoundError<'a> {
    #[error("failed to load sound\npath: {filename:?}")]
    LoadFailed { filename: &'a str },
    #[error("failed to load sound from wave")]
    LoadFromWaveFailed,
    #[error("cannot load wave\npath: {filename:?}")]
    LoadWaveFromFileFailed { filename: &'a str },
    #[error("wave data is null, check provided buffer data")]
    Null,
    #[error("music could not be loaded from file\npath: {filename:?}")]
    LoadMusicFromFileFailed { filename: &'a str },
    #[error("music's buffer data data is null, check provided buffer data")]
    MusicNull,
}

#[derive(Error, Debug)]
pub enum RaylibAllocationError {
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
pub enum RaylibCompressionError {
    #[error("could not compress data")]
    CompressionFailed,
}

#[derive(Error, Debug)]
pub enum RaylibLoadModelError<'a> {
    #[error("could not load model\npath: {filename:?}")]
    LoadFromFileFailed { filename: &'a str },
    #[error("could not load model from mesh")]
    LoadFromMeshFailed,
}

#[derive(Error, Debug)]
pub enum RaylibLoadModelAnimError<'a> {
    #[error("no model animations loaded\npath: {filename:?}")]
    NoAnimationsLoaded { filename: &'a str },
}

#[derive(Error, Debug)]
pub enum RaylibSetMaterialError {
    #[error("mesh_id greater than mesh count")]
    MeshIdOutOfBounds,
    #[error("material_id greater than material count")]
    MaterialIdOutOfBounds,
}

#[derive(Error, Debug)]
pub enum RaylibLoadMaterialError<'a> {
    #[error("no materials loaded\npath: {filename:?}")]
    NoneLoaded { filename: &'a str },
}

#[derive(Error, Debug)]
pub enum RaylibLoadFontError<'a> {
    #[error("error loading font; check if the file exists and if it's the right type\npath: {filename:?}")]
    LoadFromFileFailed { filename: &'a str },
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
pub enum RaylibUpdateTextureError {
    #[error("data is wrong size\nexpected {expect} bytes, got {actual}")]
    WrongDataSize { expect: usize, actual: usize },
    #[error("destination rectangle cannot exceed texture bounds")]
    OutOfBounds,
    #[error("destination rectangle cannot have negative extents")]
    NegativeSize,
}

#[derive(Error, Debug)]
pub enum RaylibLoadTextureError<'a> {
    #[error("failed to load the texture\npath: {filename:?}")]
    TextureFromFileFailed { filename: &'a str },
    #[error("failed to load image as a texture cubemap")]
    CubemapFromImageFailed,
    #[error("failed to load image as a texture")]
    TextureFromImageFailed,
    #[error("failed to create render texture")]
    CreateRenderTextureFailed,
}
