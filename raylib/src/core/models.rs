//! 3D Model, Mesh, and Animation

use crate::MintVec3;
use crate::core::math::BoundingBox;
use crate::core::math::Matrix;
use crate::core::math::Transform;
use crate::core::math::Vector3;
use crate::core::texture::Image;
use crate::core::{RaylibHandle, RaylibThread};
use crate::ffi::Color;
use crate::{
    consts,
    error::{LoadMaterialError, LoadModelAnimError, LoadModelError, SetMaterialError},
    ffi,
};
use std::ffi::CString;

fn no_drop<T>(_thing: T) {}
make_thin_wrapper!(
    /// Model, meshes, materials and animation data
    Model,
    ffi::Model,
    ffi::UnloadModel
);
make_thin_wrapper!(
    /// Unowned version of [`Model`] that does not free the resource when dropped
    WeakModel,
    ffi::Model,
    no_drop
);
make_thin_wrapper!(
    /// Mesh, vertex data and vao/vbo
    Mesh,
    ffi::Mesh,
    |mesh: ffi::Mesh| ffi::UnloadMesh(mesh)
);
make_thin_wrapper!(
    /// Unowned version of [`Mesh`] that does not free the resource when dropped
    WeakMesh,
    ffi::Mesh,
    no_drop
);
make_thin_wrapper!(
    /// Material, includes shader and maps
    Material,
    ffi::Material,
    ffi::UnloadMaterial
);
make_thin_wrapper!(
    /// Unowned version of [`Material`] that does not free the resource when dropped
    WeakMaterial,
    ffi::Material,
    no_drop
);
make_thin_wrapper!(
    /// Bone, skeletal animation bone
    BoneInfo,
    ffi::BoneInfo,
    no_drop
);
make_thin_wrapper!(
    /// [`ModelAnimation`]
    ModelAnimation,
    ffi::ModelAnimation,
    ffi::UnloadModelAnimation
);
make_thin_wrapper!(
    /// Unowned version of [`ModelAnimation`] that does not free the resource when dropped
    WeakModelAnimation,
    ffi::ModelAnimation,
    no_drop
);
make_thin_wrapper!(
    /// [`MaterialMap`]
    MaterialMap,
    ffi::MaterialMap,
    no_drop
);

// Weak things can be clone
impl Clone for WeakModel {
    fn clone(&self) -> WeakModel {
        WeakModel(self.0)
    }
}

// Weak things can be clone
impl Clone for WeakMesh {
    fn clone(&self) -> WeakMesh {
        WeakMesh(self.0)
    }
}

// Weak things can be clone
impl Clone for WeakMaterial {
    fn clone(&self) -> WeakMaterial {
        WeakMaterial(self.0)
    }
}

// Weak things can be clone
impl Clone for WeakModelAnimation {
    fn clone(&self) -> WeakModelAnimation {
        WeakModelAnimation(self.0)
    }
}

mod sealed {
    use super::{Mesh, WeakMesh};
    /// Convertible to weak version of `Self`
    pub trait MakeWeak {
        /// The weak form of `Self`
        type Weak;
        /// Convert `self` to its weak form, allowing it to be shared by multiple containers on the condition
        /// that it is only unloaded once, manually.
        ///
        /// # Safety
        ///
        /// Must manually free memory by calling the proper unload function.
        /// Even if the return implements [`Copy`], exactly one instance should be unloaded to avoid double-free,
        /// and copies must not be used after being unloaded to avoid use-after-free.
        #[must_use]
        unsafe fn make_weak(self) -> Self::Weak;
    }

    /// Convertible from weak version of `Self`
    pub trait FromWeak<T> {
        /// Converts weak to a "safe" version.
        ///
        /// # Safety
        ///
        /// Make sure to call this function from the thread the resource was created.
        /// If there are any aliases to `val`, they must not be used after this function's return drops,
        /// and Rust's aliasing rules must be ensured by the caller.
        #[must_use]
        unsafe fn from_weak(val: T) -> Self;
    }
    macro_rules! imlp_make_weak {
        ($($Ty:ident -> $Weak:ident;)*) => {$(
            impl MakeWeak for $Ty {
                type Weak = $Weak;
                #[inline]
                unsafe fn make_weak(self) -> Self::Weak {
                    unsafe { self.make_weak() }
                }
            }
            impl MakeWeak for $Weak {
                type Weak = $Weak;
                #[inline]
                unsafe fn make_weak(self) -> Self::Weak {
                    self
                }
            }
            impl FromWeak<$Weak> for $Ty {
                #[inline]
                unsafe fn from_weak(val: $Weak) -> Self {
                    unsafe { $Ty::from_raw(val.to_raw()) }
                }
            }
            impl FromWeak<$Ty> for $Ty {
                #[inline]
                unsafe fn from_weak(val: $Ty) -> Self {
                    val
                }
            }
        )*};
    }
    imlp_make_weak! {
        Mesh -> WeakMesh;
    }
}
pub use sealed::{FromWeak, MakeWeak};

impl RaylibHandle {
    /// Loads model from files ([`Mesh`] and [`Material`]).
    ///
    /// # Errors
    ///
    /// This function returns [`LoadModelError::LoadFromFileFailed`] if the model returned by [`ffi::LoadModel`] contains null
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte
    pub fn load_model(
        &mut self,
        _: &RaylibThread,
        filename: &str,
    ) -> Result<Model, LoadModelError> {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        let m = unsafe { ffi::LoadModel(c_filename.as_ptr()) };
        if m.meshes.is_null() && m.materials.is_null() && m.bones.is_null() && m.bindPose.is_null()
        {
            return Err(LoadModelError::LoadFromFileFailed {
                path: filename.into(),
            });
        }
        // TODO check if null pointer checks are necessary.
        Ok(Model(m))
    }

    /// Loads model from a generated [`Mesh`]
    ///
    /// # Errors
    ///
    /// This method returns [`LoadModelError::LoadFromMeshFailed`] if the model returned by [`ffi::LoadModelFromMesh`] contains null
    pub fn load_model_from_mesh(
        &mut self,
        _: &RaylibThread,
        mesh: impl MakeWeak<Weak = WeakMesh>,
    ) -> Result<Model, LoadModelError> {
        let weak_mesh = unsafe { mesh.make_weak() };
        let m = unsafe { ffi::LoadModelFromMesh(weak_mesh.0) };

        if m.meshes.is_null() || m.materials.is_null() {
            return Err(LoadModelError::LoadFromMeshFailed);
        }

        Ok(Model(m))
    }

    /// Load model animations from file
    ///
    /// # Errors
    ///
    /// This method returns [`LoadModelAnimError::NoAnimationsLoaded`] if [`ffi::LoadModelAnimations`] assigns `m_size` with a value <= 0.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    pub fn load_model_animations(
        &mut self,
        _: &RaylibThread,
        filename: &str,
    ) -> Result<Vec<ModelAnimation>, LoadModelAnimError> {
        let c_filename =
            CString::new(filename).expect("filename should not contain internal 0 byte");
        let mut m_size = 0;
        let m_ptr = unsafe { ffi::LoadModelAnimations(c_filename.as_ptr(), &raw mut m_size) };
        if let Ok(m_size @ 1..) = usize::try_from(m_size) {
            let m_arr = unsafe { std::slice::from_raw_parts_mut(m_ptr, m_size) };
            let m_vec = m_arr.iter().copied().map(ModelAnimation).collect();
            unsafe {
                ffi::MemFree(m_ptr.cast());
            }
            Ok(m_vec)
        } else {
            Err(LoadModelAnimError::NoAnimationsLoaded {
                path: filename.into(),
            })
        }
    }

    /// Update model animation pose (CPU)
    #[inline]
    pub fn update_model_animation(
        &mut self,
        _: &RaylibThread,
        mut model: impl AsMut<ffi::Model>,
        anim: impl AsRef<ffi::ModelAnimation>,
        frame: i32,
    ) {
        unsafe {
            ffi::UpdateModelAnimation(*model.as_mut(), *anim.as_ref(), frame);
        }
    }

    /// Update model animation mesh bone matrices (GPU skinning)
    #[inline]
    pub fn update_model_animation_bones(
        &mut self,
        _: &RaylibThread,
        mut model: impl AsMut<ffi::Model>,
        anim: impl AsRef<ffi::ModelAnimation>,
        frame: i32,
    ) {
        unsafe {
            ffi::UpdateModelAnimationBones(*model.as_mut(), *anim.as_ref(), frame);
        }
    }
}

impl RaylibModel for WeakModel {}
impl RaylibModel for Model {}

impl Model {
    /// Convert `self` to its weak form, allowing it to be shared by multiple containers on the condition
    /// that it is only unloaded once, manually.
    ///
    /// # Safety
    ///
    /// Must manually free memory by calling the proper unload function.
    /// Even if the return implements [`Copy`], exactly one instance should be unloaded to avoid double-free,
    /// and copies must not be used after being unloaded to avoid use-after-free.
    #[inline]
    #[must_use]
    pub const unsafe fn make_weak(self) -> WeakModel {
        let m = WeakModel(self.0);
        std::mem::forget(self);
        m
    }
}

/// [`Model`] accessors and helper methods.
pub trait RaylibModel {
    /// Local transform matrix
    #[inline]
    #[must_use]
    fn transform(&self) -> &Matrix
    where
        Self: AsRef<ffi::Model>,
    {
        unsafe { &*(std::ptr::from_ref(&self.as_ref().transform).cast()) }
    }

    /// Set the local transformation matrix
    #[inline]
    fn set_transform(&mut self, mat: &Matrix)
    where
        Self: AsMut<ffi::Model>,
    {
        self.as_mut().transform = (*mat).into();
    }

    /// Meshes array
    #[inline]
    #[must_use]
    fn meshes(&self) -> &[WeakMesh]
    where
        Self: AsRef<ffi::Model>,
    {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().meshes.cast(),
                self.as_ref()
                    .meshCount
                    .try_into()
                    .expect("meshCount should not be negative"),
            )
        }
    }

    /// Meshes array
    #[inline]
    #[must_use]
    fn meshes_mut(&mut self) -> &mut [WeakMesh]
    where
        Self: AsMut<ffi::Model>,
    {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().meshes.cast(),
                self.as_mut()
                    .meshCount
                    .try_into()
                    .expect("meshCount should not be negative"),
            )
        }
    }

    /// Materials array
    #[inline]
    #[must_use]
    fn materials(&self) -> &[WeakMaterial]
    where
        Self: AsRef<ffi::Model>,
    {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().materials.cast(),
                self.as_ref()
                    .materialCount
                    .try_into()
                    .expect("materialCount should not be negative"),
            )
        }
    }

    /// Materials array
    #[inline]
    #[must_use]
    fn materials_mut(&mut self) -> &mut [WeakMaterial]
    where
        Self: AsMut<ffi::Model>,
    {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().materials.cast(),
                self.as_mut()
                    .materialCount
                    .try_into()
                    .expect("materialCount should not be negative"),
            )
        }
    }

    /// Bones information (skeleton)
    #[inline]
    #[must_use]
    fn bones(&self) -> Option<&[BoneInfo]>
    where
        Self: AsRef<ffi::Model>,
    {
        if self.as_ref().bones.is_null() {
            return None;
        }

        Some(unsafe {
            std::slice::from_raw_parts(
                self.as_ref().bones.cast(),
                self.as_ref()
                    .boneCount
                    .try_into()
                    .expect("boneCount should not be negative"),
            )
        })
    }

    /// Bones information (skeleton)
    #[inline]
    #[must_use]
    fn bones_mut(&mut self) -> Option<&mut [BoneInfo]>
    where
        Self: AsMut<ffi::Model>,
    {
        if self.as_mut().bones.is_null() {
            return None;
        }

        Some(unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().bones.cast(),
                self.as_mut()
                    .boneCount
                    .try_into()
                    .expect("boneCount should not be negative"),
            )
        })
    }

    /// Bones base transformation (pose)
    #[inline]
    #[must_use]
    fn bind_pose(&self) -> Option<&Transform>
    where
        Self: AsRef<ffi::Model>,
    {
        if self.as_ref().bindPose.is_null() {
            return None;
        }
        Some(unsafe { &*(self.as_ref().bindPose.cast()) })
    }

    /// Bones base transformation (pose)
    #[inline]
    #[must_use]
    fn bind_pose_mut(&mut self) -> Option<&mut Transform>
    where
        Self: AsMut<ffi::Model>,
    {
        if self.as_mut().bindPose.is_null() {
            return None;
        }
        Some(unsafe { &mut *(self.as_mut().bindPose.cast()) })
    }

    /// Check model animation skeleton match
    #[inline]
    #[must_use]
    fn is_model_animation_valid(&self, anim: &ModelAnimation) -> bool
    where
        Self: AsRef<ffi::Model>,
    {
        unsafe { ffi::IsModelAnimationValid(*self.as_ref(), anim.0) }
    }

    /// Check if a model is ready
    #[inline]
    #[must_use]
    fn is_model_valid(&self) -> bool
    where
        Self: AsRef<ffi::Model>,
    {
        unsafe { ffi::IsModelValid(*self.as_ref()) }
    }

    /// Compute model bounding box limits (considers all meshes)
    #[inline]
    #[must_use]
    fn get_model_bounding_box(&self) -> BoundingBox
    where
        Self: AsRef<ffi::Model>,
    {
        unsafe { BoundingBox::from(ffi::GetModelBoundingBox(*self.as_ref())) }
    }

    /// Set material for a mesh
    ///
    /// # Errors
    ///
    /// This method returns [`SetMaterialError::MeshIdOutOfBounds`] if `mesh_id` is greater than `meshCount`,
    /// or [`SetMaterialError::MaterialIdOutOfBounds`] if `material_id` is greater than `materialCount`.
    #[inline]
    fn set_model_mesh_material(
        &mut self,
        mesh_id: i32,
        material_id: i32,
    ) -> Result<(), SetMaterialError>
    where
        Self: AsMut<ffi::Model>,
    {
        // should this be an assertion?
        if mesh_id >= self.as_mut().meshCount {
            Err(SetMaterialError::MeshIdOutOfBounds)
        } else if material_id >= self.as_mut().materialCount {
            Err(SetMaterialError::MaterialIdOutOfBounds)
        } else {
            unsafe { ffi::SetModelMeshMaterial(self.as_mut(), mesh_id, material_id) };
            Ok(())
        }
    }
}

impl RaylibMesh for WeakMesh {}
impl RaylibMesh for Mesh {}

impl Mesh {
    /// Convert `self` to its weak form, allowing it to be shared by multiple containers on the condition
    /// that it is only unloaded once, manually.
    ///
    /// # Safety
    ///
    /// Must manually free memory by calling the proper unload function.
    /// Even if the return implements [`Copy`], exactly one instance should be unloaded to avoid double-free,
    /// and copies must not be used after being unloaded to avoid use-after-free.
    #[inline]
    #[must_use]
    pub const unsafe fn make_weak(self) -> WeakMesh {
        let m = WeakMesh(self.0);
        std::mem::forget(self);
        m
    }
}

/// [`Mesh`] accessors and helper methods.
pub trait RaylibMesh {
    /// Upload mesh vertex data in GPU and provide VAO/VBO ids
    #[inline]
    unsafe fn upload(&mut self, dynamic: bool)
    where
        Self: AsMut<ffi::Mesh>,
    {
        unsafe {
            ffi::UploadMesh(self.as_mut(), dynamic);
        }
    }

    /// Update mesh vertex data in GPU for a specific buffer index
    ///
    /// # Panics
    ///
    /// This method will panic if `data` has a length greater than [`i32::MAX`].
    #[inline]
    unsafe fn update_buffer<A>(&mut self, index: i32, data: &[u8], offset: i32)
    where
        Self: AsRef<ffi::Mesh>,
    {
        unsafe {
            ffi::UpdateMeshBuffer(
                *self.as_ref(),
                index,
                data.as_ptr().cast(),
                data.len()
                    .try_into()
                    .expect("data should not exceed i32::MAX elements"),
                offset,
            );
        }
    }

    /// Vertex position (XYZ - 3 components per vertex) (shader-location = 0)
    ///
    /// # Panics
    ///
    /// This method will panic if `vertexCount` is negative
    #[inline]
    #[must_use]
    fn vertices(&self) -> &[Vector3]
    where
        Self: AsRef<ffi::Mesh>,
    {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().vertices.cast(),
                self.as_ref()
                    .vertexCount
                    .try_into()
                    .expect("vertexCount should not be negative"),
            )
        }
    }

    /// Vertex position (XYZ - 3 components per vertex) (shader-location = 0)
    ///
    /// # Panics
    ///
    /// This method will panic if `vertexCount` is negative
    #[inline]
    #[must_use]
    fn vertices_mut(&mut self) -> &mut [Vector3]
    where
        Self: AsMut<ffi::Mesh>,
    {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().vertices.cast(),
                self.as_mut()
                    .vertexCount
                    .try_into()
                    .expect("vertexCount should not be negative"),
            )
        }
    }

    /// Vertex normals (XYZ - 3 components per vertex) (shader-location = 2)
    ///
    /// # Panics
    ///
    /// This method will panic if `vertexCount` is negative
    #[inline]
    #[must_use]
    fn normals(&self) -> &[Vector3]
    where
        Self: AsRef<ffi::Mesh>,
    {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().normals.cast(),
                self.as_ref()
                    .vertexCount
                    .try_into()
                    .expect("vertexCount should not be negative"),
            )
        }
    }

    /// Vertex normals (XYZ - 3 components per vertex) (shader-location = 2)
    ///
    /// # Panics
    ///
    /// This method will panic if `vertexCount` is negative
    #[inline]
    #[must_use]
    fn normals_mut(&mut self) -> &mut [Vector3]
    where
        Self: AsMut<ffi::Mesh>,
    {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().normals.cast(),
                self.as_mut()
                    .vertexCount
                    .try_into()
                    .expect("vertexCount should not be negative"),
            )
        }
    }

    /// Vertex tangents (XYZW - 4 components per vertex) (shader-location = 4)
    ///
    /// # Panics
    ///
    /// This method will panic if `vertexCount` is negative
    #[inline]
    #[must_use]
    fn tangents(&self) -> &[Vector3]
    where
        Self: AsRef<ffi::Mesh>,
    {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().tangents.cast(),
                self.as_ref()
                    .vertexCount
                    .try_into()
                    .expect("vertexCount should not be negative"),
            )
        }
    }

    /// Vertex tangents (XYZW - 4 components per vertex) (shader-location = 4)
    ///
    /// # Panics
    ///
    /// This method will panic if `vertexCount` is negative
    #[inline]
    #[must_use]
    fn tangents_mut(&mut self) -> &mut [Vector3]
    where
        Self: AsMut<ffi::Mesh>,
    {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().tangents.cast(),
                self.as_mut()
                    .vertexCount
                    .try_into()
                    .expect("vertexCount should not be negative"),
            )
        }
    }

    /// Vertex colors (RGBA - 4 components per vertex) (shader-location = 3)
    ///
    /// # Panics
    ///
    /// This method will panic if `vertexCount` is negative
    #[inline]
    #[must_use]
    fn colors(&self) -> &[Color]
    where
        Self: AsRef<ffi::Mesh>,
    {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().colors.cast(),
                self.as_ref()
                    .vertexCount
                    .try_into()
                    .expect("vertexCount should not be negative"),
            )
        }
    }

    /// Vertex colors (RGBA - 4 components per vertex) (shader-location = 3)
    ///
    /// # Panics
    ///
    /// This method will panic if `vertexCount` is negative
    #[inline]
    #[must_use]
    fn colors_mut(&mut self) -> &mut [Color]
    where
        Self: AsMut<ffi::Mesh>,
    {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().colors.cast(),
                self.as_mut()
                    .vertexCount
                    .try_into()
                    .expect("vertexCount should not be negative"),
            )
        }
    }

    /// Vertex indices (in case vertex data comes indexed)
    ///
    /// # Panics
    ///
    /// This method will panic if `triangleCount * 3` is negative or would overflow
    #[inline]
    #[must_use]
    fn indices(&self) -> &[u16]
    where
        Self: AsRef<ffi::Mesh>,
    {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().indices.cast(),
                self.as_ref()
                    .triangleCount
                    .try_into()
                    .ok()
                    .and_then(|n: usize| n.checked_mul(3))
                    .expect("triangleCount*3 should not be negative or overflow"),
            )
        }
    }

    /// Vertex indices (in case vertex data comes indexed)
    ///
    /// # Panics
    ///
    /// This method will panic if `triangleCount * 3` is negative or would overflow
    #[inline]
    #[must_use]
    fn indices_mut(&mut self) -> &mut [u16]
    where
        Self: AsMut<ffi::Mesh>,
    {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().indices.cast(),
                self.as_mut()
                    .triangleCount
                    .try_into()
                    .ok()
                    .and_then(|n: usize| n.checked_mul(3))
                    .expect("triangleCount*3 should not be negative or overflow"),
            )
        }
    }

    /// Generate polygonal mesh
    #[inline]
    #[must_use]
    fn gen_mesh_poly(_: &RaylibThread, sides: i32, radius: f32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshPoly(sides, radius)) }
    }

    /// Generates plane mesh (with subdivisions).
    #[inline]
    #[must_use]
    fn gen_mesh_plane(_: &RaylibThread, width: f32, length: f32, res_x: i32, res_z: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshPlane(width, length, res_x, res_z)) }
    }

    /// Generates cuboid mesh.
    #[inline]
    #[must_use]
    fn gen_mesh_cube(_: &RaylibThread, width: f32, height: f32, length: f32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshCube(width, height, length)) }
    }

    /// Generates sphere mesh (standard sphere).
    #[inline]
    #[must_use]
    fn gen_mesh_sphere(_: &RaylibThread, radius: f32, rings: i32, slices: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshSphere(radius, rings, slices)) }
    }

    /// Generates half-sphere mesh (no bottom cap).
    #[inline]
    #[must_use]
    fn gen_mesh_hemisphere(_: &RaylibThread, radius: f32, rings: i32, slices: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshHemiSphere(radius, rings, slices)) }
    }

    /// Generates cylinder mesh.
    #[inline]
    #[must_use]
    fn gen_mesh_cylinder(_: &RaylibThread, radius: f32, height: f32, slices: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshCylinder(radius, height, slices)) }
    }

    /// Generates torus mesh.
    #[inline]
    #[must_use]
    fn gen_mesh_torus(_: &RaylibThread, radius: f32, size: f32, rad_seg: i32, sides: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshTorus(radius, size, rad_seg, sides)) }
    }

    /// Generates trefoil knot mesh.
    #[inline]
    #[must_use]
    fn gen_mesh_knot(_: &RaylibThread, radius: f32, size: f32, rad_seg: i32, sides: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshKnot(radius, size, rad_seg, sides)) }
    }

    /// Generates heightmap mesh from image data.
    #[inline]
    #[must_use]
    fn gen_mesh_heightmap(_: &RaylibThread, heightmap: &Image, size: impl Into<MintVec3>) -> Mesh {
        unsafe { Mesh(ffi::GenMeshHeightmap(heightmap.0, size.into())) }
    }

    /// Generates cubes-based map mesh from image data.
    #[inline]
    #[must_use]
    fn gen_mesh_cubicmap(
        _: &RaylibThread,
        cubicmap: &Image,
        cube_size: impl Into<MintVec3>,
    ) -> Mesh {
        unsafe { Mesh(ffi::GenMeshCubicmap(cubicmap.0, cube_size.into())) }
    }

    /// Generate cone/pyramid mesh
    #[inline]
    #[must_use]
    fn gen_mesh_cone(_: &RaylibThread, radius: f32, height: f32, slices: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshCone(radius, height, slices)) }
    }

    /// Computes mesh bounding box limits.
    #[inline]
    #[must_use]
    fn get_mesh_bounding_box(&self) -> BoundingBox
    where
        Self: AsRef<ffi::Mesh>,
    {
        unsafe { ffi::GetMeshBoundingBox(*self.as_ref()).into() }
    }

    /// Computes mesh tangents.
    // NOTE: New VBO for tangents is generated at default location and also binded to mesh VAO
    #[inline]
    fn gen_mesh_tangents(&mut self, _: &RaylibThread)
    where
        Self: AsMut<ffi::Mesh>,
    {
        unsafe {
            ffi::GenMeshTangents(self.as_mut());
        }
    }

    /// Exports mesh as an OBJ file.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    #[inline]
    fn export(&self, filename: &str)
    where
        Self: AsRef<ffi::Mesh>,
    {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        unsafe {
            ffi::ExportMesh(*self.as_ref(), c_filename.as_ptr());
        }
    }

    /// Export mesh as code file (.h) defining multiple arrays of vertex attributes
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    #[inline]
    fn export_as_code(&self, filename: &str)
    where
        Self: AsRef<ffi::Mesh>,
    {
        let c_filename =
            CString::new(filename).expect("filename should not contain an internal 0 byte");
        unsafe {
            ffi::ExportMeshAsCode(*self.as_ref(), c_filename.as_ptr());
        }
    }
}

impl Material {
    /// Convert `self` to its weak form, allowing it to be shared by multiple containers on the condition
    /// that it is only unloaded once, manually.
    ///
    /// # Safety
    ///
    /// Must manually free memory by calling the proper unload function.
    /// Even if the return implements [`Copy`], exactly one instance should be unloaded to avoid double-free,
    /// and copies must not be used after being unloaded to avoid use-after-free.
    #[inline]
    #[must_use]
    pub const unsafe fn make_weak(self) -> WeakMaterial {
        let m = WeakMaterial(self.0);
        std::mem::forget(self);
        m
    }

    /// Load materials from model file
    ///
    /// # Errors
    ///
    /// This method returns [`LoadMaterialError::NoneLoaded`] if [`ffi::LoadMaterials`] outputs a size smaller than 0.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    pub fn load_materials(filename: &str) -> Result<Vec<Material>, LoadMaterialError> {
        let c_filename =
            CString::new(filename).expect("filename should not contain internal 0 byte");
        let mut m_size = 0;
        let m_ptr = unsafe { ffi::LoadMaterials(c_filename.as_ptr(), &raw mut m_size) };
        if let Ok(m_size @ 1..) = usize::try_from(m_size) {
            let m_arr = unsafe { std::slice::from_raw_parts_mut(m_ptr, m_size) };
            let m_vec = m_arr.iter().copied().map(Material).collect();
            unsafe {
                ffi::MemFree(m_ptr.cast());
            }
            Ok(m_vec)
        } else {
            Err(LoadMaterialError::NoneLoaded {
                path: filename.into(),
            })
        }
    }
}

impl RaylibMaterial for WeakMaterial {}
impl RaylibMaterial for Material {}

/// [`Material`] accessors and helper methods.
pub trait RaylibMaterial {
    /// Material shader
    #[inline]
    #[must_use]
    fn shader(&self) -> &crate::shaders::WeakShader
    where
        Self: AsRef<ffi::Material>,
    {
        unsafe { &*std::ptr::from_ref(&self.as_ref().shader).cast() }
    }

    /// Material shader
    #[inline]
    #[must_use]
    fn shader_mut(&mut self) -> &mut crate::shaders::WeakShader
    where
        Self: AsMut<ffi::Material>,
    {
        unsafe { &mut *std::ptr::from_mut(&mut self.as_mut().shader).cast() }
    }

    /// Material maps array ([`consts::MAX_MATERIAL_MAPS`])
    #[inline]
    #[must_use]
    fn maps(&self) -> &[MaterialMap]
    where
        Self: AsRef<ffi::Material>,
    {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().maps.cast(),
                consts::MAX_MATERIAL_MAPS as usize,
            )
        }
    }

    /// Material maps array ([`consts::MAX_MATERIAL_MAPS`])
    #[inline]
    #[must_use]
    fn maps_mut(&mut self) -> &mut [MaterialMap]
    where
        Self: AsMut<ffi::Material>,
    {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().maps.cast(),
                consts::MAX_MATERIAL_MAPS as usize,
            )
        }
    }

    /// Set texture for a material map type (`MATERIAL_MAP_DIFFUSE`, `MATERIAL_MAP_SPECULAR`...)
    #[inline]
    fn set_material_texture(
        &mut self,
        map_type: crate::consts::MaterialMapIndex,
        texture: impl AsRef<ffi::Texture2D>,
    ) where
        Self: AsMut<ffi::Material>,
    {
        unsafe { ffi::SetMaterialTexture(self.as_mut(), map_type as i32, *texture.as_ref()) }
    }

    /// Check if a material is valid (shader assigned, map textures loaded in GPU)
    #[inline]
    #[must_use]
    fn is_material_valid(&mut self) -> bool
    where
        Self: AsRef<ffi::Material>,
    {
        unsafe { ffi::IsMaterialValid(*self.as_ref()) }
    }
}

/// Iterator over frame pose references
///
/// Returned by [`RaylibModelAnimation::frame_poses_iter`].
#[derive(Debug, Clone)]
pub struct FramePoseIter<'a> {
    iter: std::slice::Iter<'a, Option<&'a [Transform]>>,
    bone_count: usize,
}
impl<'a> FramePoseIter<'a> {
    #[must_use]
    unsafe fn new(
        frame_poses: *mut *mut ffi::Transform,
        frame_count: usize,
        bone_count: usize,
    ) -> Self {
        // No new items are being created that get dropped here, these are just changes in perspective of how to borrow-check the pointers.
        assert!(!frame_poses.is_null(), "frame pose array cannot be null");
        assert!(frame_poses.is_aligned(), "frame pose array must be aligned");
        let frame_poses = frame_poses.cast::<Option<&'a [Transform]>>();
        let iter = unsafe { std::slice::from_raw_parts(frame_poses, frame_count) }.iter();
        Self { iter, bone_count }
    }
    const fn func(tf: Option<&'a [Transform]>, bone_count: usize) -> &'a [Transform] {
        unsafe {
            std::slice::from_raw_parts(
                tf.expect("frame pose transform cannot be null").as_ptr(),
                bone_count,
            )
        }
    }
}
impl<'a> Iterator for FramePoseIter<'a> {
    type Item = &'a [Transform];

    fn next(&mut self) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter
            .next()
            .copied()
            .map(move |tf| Self::func(tf, bone_count))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    fn last(self) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter
            .last()
            .copied()
            .map(move |tf| Self::func(tf, bone_count))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter
            .nth(n)
            .copied()
            .map(move |tf| Self::func(tf, bone_count))
    }
}
impl DoubleEndedIterator for FramePoseIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter
            .next_back()
            .copied()
            .map(move |tf| Self::func(tf, bone_count))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter
            .nth_back(n)
            .copied()
            .map(move |tf| Self::func(tf, bone_count))
    }
}
impl ExactSizeIterator for FramePoseIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// Iterator over mutable frame pose references
///
/// Returned by [`RaylibModelAnimation::frame_poses_iter`].
#[derive(Debug)]
pub struct FramePoseIterMut<'a> {
    iter: std::slice::IterMut<'a, Option<&'a mut [Transform]>>,
    bone_count: usize,
}
impl<'a> FramePoseIterMut<'a> {
    unsafe fn new(
        frame_poses: *mut *mut ffi::Transform,
        frame_count: usize,
        bone_count: usize,
    ) -> Self {
        // No new items are being created that get dropped here, these are just changes in perspective of how to borrow-check the pointers.
        assert!(!frame_poses.is_null(), "frame pose array cannot be null");
        assert!(frame_poses.is_aligned(), "frame pose array must be aligned");
        let frame_poses = frame_poses.cast::<Option<&'a mut [Transform]>>();
        let iter = unsafe { std::slice::from_raw_parts_mut(frame_poses, frame_count) }.iter_mut();
        Self { iter, bone_count }
    }
    const fn func(tf: &mut Option<&'a mut [Transform]>, bone_count: usize) -> &'a mut [Transform] {
        unsafe {
            std::slice::from_raw_parts_mut(
                tf.as_mut()
                    .expect("frame pose transform cannot be null")
                    .as_mut_ptr(),
                bone_count,
            )
        }
    }
}
impl<'a> Iterator for FramePoseIterMut<'a> {
    type Item = &'a mut [Transform];

    fn next(&mut self) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter.next().map(move |tf| Self::func(tf, bone_count))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    fn last(self) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter.last().map(move |tf| Self::func(tf, bone_count))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter.nth(n).map(move |tf| Self::func(tf, bone_count))
    }
}
impl DoubleEndedIterator for FramePoseIterMut<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter
            .next_back()
            .map(move |tf| Self::func(tf, bone_count))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter
            .nth_back(n)
            .map(move |tf| Self::func(tf, bone_count))
    }
}
impl ExactSizeIterator for FramePoseIterMut<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl RaylibModelAnimation for ModelAnimation {}
impl RaylibModelAnimation for WeakModelAnimation {}

impl ModelAnimation {
    /// Convert `self` to its weak form, allowing it to be shared by multiple containers on the condition
    /// that it is only unloaded once, manually.
    ///
    /// # Safety
    ///
    /// Must manually free memory by calling the proper unload function.
    /// Even if the return implements [`Copy`], exactly one instance should be unloaded to avoid double-free,
    /// and copies must not be used after being unloaded to avoid use-after-free.
    #[inline]
    #[must_use]
    pub const unsafe fn make_weak(self) -> WeakModelAnimation {
        let m = WeakModelAnimation(self.0);
        std::mem::forget(self);
        m
    }
}

/// [`ModelAnimation`] accessors and helper methods.
pub trait RaylibModelAnimation {
    /// Bones information (skeleton)
    ///
    /// # Panics
    ///
    /// This method will panic if `boneCount` is negative
    #[inline]
    #[must_use]
    fn bones(&self) -> &[BoneInfo]
    where
        Self: AsRef<ffi::ModelAnimation>,
    {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().bones as *const BoneInfo,
                self.as_ref()
                    .boneCount
                    .try_into()
                    .expect("boneCount should not be negative"),
            )
        }
    }

    /// Bones information (skeleton)
    ///
    /// # Panics
    ///
    /// This method will panic if `boneCount` is negative
    #[inline]
    #[must_use]
    fn bones_mut(&mut self) -> &mut [BoneInfo]
    where
        Self: AsMut<ffi::ModelAnimation>,
    {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().bones.cast(),
                self.as_mut()
                    .boneCount
                    .try_into()
                    .expect("boneCount should not be negative"),
            )
        }
    }

    /// Poses array by frame
    ///
    /// # Panics
    ///
    /// This method will panic if `frameCount` or `boneCount` is negative
    #[must_use]
    fn frame_poses(&self) -> Vec<&[Transform]>
    where
        Self: AsRef<ffi::ModelAnimation>,
    {
        let anim = self.as_ref();
        let bone_count = anim
            .boneCount
            .try_into()
            .expect("boneCount should not be negative");

        unsafe {
            std::slice::from_raw_parts(
                anim.framePoses.cast::<*const Transform>(),
                anim.frameCount
                    .try_into()
                    .expect("frameCount should not be negative"),
            )
        }
        .iter()
        .copied()
        .map(|frame_pose| unsafe { std::slice::from_raw_parts(frame_pose, bone_count) })
        .collect()
    }

    /// Iterator over poses array by frame
    ///
    /// # Panics
    ///
    /// This method will panic if `frameCount` or `boneCount` is negative
    #[must_use]
    fn frame_poses_iter(&self) -> FramePoseIter<'_>
    where
        Self: AsRef<ffi::ModelAnimation>,
    {
        let anim = self.as_ref();
        unsafe {
            FramePoseIter::new(
                anim.framePoses,
                anim.frameCount
                    .try_into()
                    .expect("frameCount should not be negative"),
                anim.boneCount
                    .try_into()
                    .expect("boneCount should not be negative"),
            )
        }
    }

    /// Poses array by frame
    ///
    /// # Panics
    ///
    /// This method will panic if `frameCount` or `boneCount` is negative
    #[must_use]
    fn frame_poses_mut(&mut self) -> Vec<&mut [Transform]>
    where
        Self: AsMut<ffi::ModelAnimation>,
    {
        let anim = self.as_mut();
        let bone_count = anim
            .boneCount
            .try_into()
            .expect("boneCount should not be negative");

        unsafe {
            std::slice::from_raw_parts_mut(
                anim.framePoses.cast::<*mut Transform>(),
                anim.frameCount
                    .try_into()
                    .expect("frameCount should not be negative"),
            )
        }
        .iter()
        .copied()
        .map(|frame_pose| unsafe { std::slice::from_raw_parts_mut(frame_pose, bone_count) })
        .collect()
    }

    /// Iterator over poses array by frame
    ///
    /// # Panics
    ///
    /// This method will panic if `frameCount` or `boneCount` is negative
    #[must_use]
    fn frame_poses_iter_mut(&mut self) -> FramePoseIterMut<'_>
    where
        Self: AsMut<ffi::ModelAnimation>,
    {
        let anim = self.as_mut();
        unsafe {
            FramePoseIterMut::new(
                anim.framePoses,
                anim.frameCount
                    .try_into()
                    .expect("frameCount should not be negative"),
                anim.boneCount
                    .try_into()
                    .expect("boneCount should not be negative"),
            )
        }
    }
}

impl MaterialMap {
    /// Material map texture
    #[inline]
    #[must_use]
    pub const fn texture(&self) -> &crate::texture::WeakTexture2D {
        unsafe { &*std::ptr::from_ref(&self.0.texture).cast() }
    }

    /// Material map texture
    #[inline]
    #[must_use]
    pub const fn texture_mut(&mut self) -> &mut crate::texture::WeakTexture2D {
        unsafe { &mut *std::ptr::from_mut(&mut self.0.texture).cast() }
    }

    /// Material map color
    #[inline]
    #[must_use]
    pub const fn color(&self) -> &Color {
        unsafe { &*std::ptr::from_ref(&self.0.color).cast() }
    }

    /// Material map color
    #[inline]
    #[must_use]
    pub const fn color_mut(&mut self) -> &mut Color {
        unsafe { &mut *std::ptr::from_mut(&mut self.0.color).cast() }
    }

    /// Material map value
    #[inline]
    #[must_use]
    pub const fn value(&self) -> &f32 {
        unsafe { &*std::ptr::from_ref(&self.0.value).cast() }
    }

    /// Material map value
    #[inline]
    #[must_use]
    pub const fn value_mut(&mut self) -> &mut f32 {
        unsafe { &mut *std::ptr::from_mut(&mut self.0.value).cast() }
    }
}

impl RaylibHandle {
    /// Load default material (Supports: `DIFFUSE`, `SPECULAR`, `NORMAL` maps)
    #[inline]
    #[must_use]
    pub fn load_material_default(&self, _: &RaylibThread) -> WeakMaterial {
        WeakMaterial(unsafe { ffi::LoadMaterialDefault() })
    }

    /// Unload material from GPU memory (VRAM)
    ///
    /// Weak materials will leak memory if they are not unlaoded
    ///
    /// # Safety
    ///
    /// This method frees the resource associated with `material`.
    /// The caller must ensure that `material` has not yet been unloaded, and that no copies
    /// of `material` are accessed or unloaded after this method returns.
    #[inline]
    pub unsafe fn unload_material(&mut self, _: &RaylibThread, material: WeakMaterial) {
        unsafe { ffi::UnloadMaterial(material.to_raw()) }
    }

    /// Unload model from GPU memory (VRAM)
    ///
    /// Weak models will leak memory if they are not unlaoded
    ///
    /// # Safety
    ///
    /// This method frees the resource associated with `model`.
    /// The caller must ensure that `model` has not yet been unloaded, and that no copies
    /// of `model` are accessed or unloaded after this method returns.
    #[inline]
    pub unsafe fn unload_model(&mut self, _: &RaylibThread, model: WeakModel) {
        unsafe { ffi::UnloadModel(model.to_raw()) }
    }

    /// Weak model animations will leak memory if they are not unlaoded
    /// Unload model animation from GPU memory (VRAM)
    ///
    /// # Safety
    ///
    /// This method frees the resource associated with `model_animation`.
    /// The caller must ensure that `model_animation` has not yet been unloaded, and that no copies
    /// of `model_animation` are accessed or unloaded after this method returns.
    #[inline]
    pub unsafe fn unload_model_animation(
        &mut self,
        _: &RaylibThread,
        model_animation: WeakModelAnimation,
    ) {
        unsafe { ffi::UnloadModelAnimation(model_animation.to_raw()) }
    }

    /// Unload mesh from GPU memory (VRAM)
    ///
    /// Weak meshes will leak memory if they are not unlaoded
    ///
    /// # Safety
    ///
    /// This method frees the resource associated with `mesh`.
    /// The caller must ensure that `mesh` has not yet been unloaded, and that no copies
    /// of `mesh` are accessed or unloaded after this method returns.
    #[inline]
    pub unsafe fn unload_mesh(&mut self, _: &RaylibThread, mesh: WeakMesh) {
        unsafe { ffi::UnloadMesh(mesh.to_raw()) }
    }
}
