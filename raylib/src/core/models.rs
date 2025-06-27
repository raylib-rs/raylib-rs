//! 3D Model, Mesh, and Animation

use crate::MintVec3;
use crate::core::math::BoundingBox;
use crate::core::math::Matrix;
use crate::core::math::Transform;
use crate::core::math::{Vector3, Vector4};
use crate::core::texture::Image;
use crate::core::{RaylibHandle, RaylibThread};
use crate::ffi::Color;
use crate::{
    consts,
    error::{LoadMaterialError, LoadModelAnimError, LoadModelError, SetMaterialError},
    ffi,
};
use std::ffi::CString;
use std::num::NonZeroU32;
use std::os::raw::c_void;
use std::ptr::NonNull;

fn no_drop<T>(_thing: T) {}
make_thin_wrapper!(
    /// Model, meshes, materials and animation data
    Model,
    ffi::Model,
    ffi::UnloadModel
);
make_thin_wrapper!(WeakModel, ffi::Model, no_drop);
make_thin_wrapper!(
    /// Material, includes shader and maps
    Material,
    ffi::Material,
    ffi::UnloadMaterial
);
make_thin_wrapper!(WeakMaterial, ffi::Material, no_drop);
make_thin_wrapper!(
    /// Bone, skeletal animation bone
    BoneInfo,
    ffi::BoneInfo,
    no_drop
);
make_thin_wrapper!(
    /// ModelAnimation
    ModelAnimation,
    ffi::ModelAnimation,
    ffi::UnloadModelAnimation
);
make_thin_wrapper!(WeakModelAnimation, ffi::ModelAnimation, no_drop);
make_thin_wrapper!(
    /// MaterialMap
    MaterialMap,
    ffi::MaterialMap,
    no_drop
);

#[repr(C)]
pub struct MeshVboId {
    position: NonZeroU32,
    texcoord: NonZeroU32,
    normal: NonZeroU32,
    color: NonZeroU32,
    tangent: NonZeroU32,
    texcoord2: NonZeroU32,
    indices: NonZeroU32,
    // boneids: NonZeroU32,
    // boneweights: NonZeroU32,
}

const _: () = {
    assert!(std::mem::offset_of!(MeshVboId, position) == ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_POSITION as usize * size_of::<u32>());
    assert!(std::mem::offset_of!(MeshVboId, texcoord) == ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_TEXCOORD as usize * size_of::<u32>());
    assert!(std::mem::offset_of!(MeshVboId, normal) == ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_NORMAL as usize * size_of::<u32>());
    assert!(std::mem::offset_of!(MeshVboId, color) == ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_COLOR as usize * size_of::<u32>());
    assert!(std::mem::offset_of!(MeshVboId, tangent) == ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_TANGENT as usize * size_of::<u32>());
    assert!(std::mem::offset_of!(MeshVboId, texcoord2) == ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_TEXCOORD2 as usize * size_of::<u32>());
    assert!(std::mem::offset_of!(MeshVboId, indices) == ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_INDICES as usize * size_of::<u32>());
    assert!(size_of::<MeshVboId>() == size_of::<[u32; 7]>());
    // if RL_SUPPORT_MESH_GPU_SKINNING
    // assert!(std::mem::offset_of!(MeshVboId, boneids) == ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_BONEIDS as usize * size_of::<u32>());
    // assert!(std::mem::offset_of!(MeshVboId, boneweights) == ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_BONEWEIGHTS as usize * size_of::<u32>());
};

#[repr(C)]
#[derive(Debug)]
pub struct Mesh {
    vertex_count: i32,
    triangle_count: i32,
    vertices: NonNull<f32>,
    texcoords: NonNull<f32>,
    texcoords2: NonNull<f32>,
    normals: NonNull<f32>,
    tangents: NonNull<f32>,
    colors: NonNull<u8>,
    indices: NonNull<u16>,
    anim_vertices: NonNull<f32>,
    anim_normals: NonNull<f32>,
    bone_ids: NonNull<u8>,
    bone_weights: NonNull<f32>,
    bone_matrices: NonNull<ffi::Matrix>,
    bone_count: i32,
    vao_id: NonZeroU32,
    vbo_id: NonNull<MeshVboId>,
}

const _: () = {
    assert!(std::mem::offset_of!(Mesh, vertex_count) == std::mem::offset_of!(ffi::Mesh, vertexCount));
    assert!(std::mem::offset_of!(Mesh, triangle_count) == std::mem::offset_of!(ffi::Mesh, triangleCount));
    assert!(std::mem::offset_of!(Mesh, vertices) == std::mem::offset_of!(ffi::Mesh, vertices));
    assert!(std::mem::offset_of!(Mesh, texcoords) == std::mem::offset_of!(ffi::Mesh, texcoords));
    assert!(std::mem::offset_of!(Mesh, texcoords2) == std::mem::offset_of!(ffi::Mesh, texcoords2));
    assert!(std::mem::offset_of!(Mesh, normals) == std::mem::offset_of!(ffi::Mesh, normals));
    assert!(std::mem::offset_of!(Mesh, tangents) == std::mem::offset_of!(ffi::Mesh, tangents));
    assert!(std::mem::offset_of!(Mesh, colors) == std::mem::offset_of!(ffi::Mesh, colors));
    assert!(std::mem::offset_of!(Mesh, indices) == std::mem::offset_of!(ffi::Mesh, indices));
    assert!(std::mem::offset_of!(Mesh, anim_vertices) == std::mem::offset_of!(ffi::Mesh, animVertices));
    assert!(std::mem::offset_of!(Mesh, anim_normals) == std::mem::offset_of!(ffi::Mesh, animNormals));
    assert!(std::mem::offset_of!(Mesh, bone_ids) == std::mem::offset_of!(ffi::Mesh, boneIds));
    assert!(std::mem::offset_of!(Mesh, bone_weights) == std::mem::offset_of!(ffi::Mesh, boneWeights));
    assert!(std::mem::offset_of!(Mesh, bone_matrices) == std::mem::offset_of!(ffi::Mesh, boneMatrices));
    assert!(std::mem::offset_of!(Mesh, bone_count) == std::mem::offset_of!(ffi::Mesh, boneCount));
    assert!(std::mem::offset_of!(Mesh, vao_id) == std::mem::offset_of!(ffi::Mesh, vaoId));
    assert!(std::mem::offset_of!(Mesh, vbo_id) == std::mem::offset_of!(ffi::Mesh, vboId));
    assert!(std::mem::size_of::<Mesh>() == std::mem::size_of::<ffi::Mesh>());
};

impl Drop for Mesh {
    fn drop(&mut self) {
        // SAFETY: `mesh` will not be used after `self` drops.
        let mesh = unsafe { self.make_raw() };
        // SAFETY: Mesh must be valid if constructed
        unsafe { ffi::UnloadMesh(mesh) }
    }
}

impl Mesh {
    /// Casts a strong [`Mesh`] reference to a weak [`ffi::Mesh`] reference.
    ///
    /// # Safety
    ///
    /// Do not mutate the data pointed to by any field of the returned reference in
    /// such a way that it would no longer be valid.
    pub const unsafe fn as_raw(&self) -> &ffi::Mesh {
        // SAFETY: `Mesh` has the same size, fields, and layout as `ffi::Mesh`.
        unsafe { std::mem::transmute(self) }
    }

    /// Casts a strong [`Mesh`] mutable reference to a weak [`ffi::Mesh`] mutable reference.
    ///
    /// # Safety
    ///
    /// Do not mutate any field of the returned reference in such a way that it would
    /// no longer be valid.
    pub const unsafe fn as_raw_mut(&mut self) -> &mut ffi::Mesh {
        // SAFETY: `Mesh` has the same size, fields, and layout as `ffi::Mesh`.
        unsafe { std::mem::transmute(self) }
    }

    /// Returns a raylib-sys mesh referring to the same resource as `self`.
    ///
    /// # Safety
    ///
    /// The returned mesh is a **weak, shallow copy**. Dropping it will not free the
    /// resources associated with `self`. When `self` is dropped, the weak copy will
    /// contain dangling pointers, which will cause UB if dereferenced.
    ///
    /// The returned mesh must not be used after any copy of `self` is released.
    pub const unsafe fn make_raw(&self) -> ffi::Mesh {
        unsafe { *self.as_raw() }
    }

    /// Converts raylib-sys object to a "safe" version.
    ///
    /// # Safety
    ///
    /// Make sure to call this function from the thread the resource was created.
    ///
    /// The mesh resource that `raw` represents must be released **exactly once**.
    /// It is strongly recommended to avoid having multiple [`Mesh`]es referring
    /// to the same mesh resource, for the sake of sanity.
    pub const unsafe fn from_raw(raw: ffi::Mesh) -> Option<Self> {
        if
            raw.vertexCount >= 0 &&
            raw.triangleCount >= 0 &&
            !raw.vertices.is_null() &&
            !raw.texcoords.is_null() &&
            !raw.texcoords2.is_null() &&
            !raw.normals.is_null() &&
            !raw.tangents.is_null() &&
            !raw.colors.is_null() &&
            !raw.indices.is_null() &&
            !raw.animVertices.is_null() &&
            !raw.animNormals.is_null() &&
            !raw.boneIds.is_null() &&
            !raw.boneWeights.is_null() &&
            !raw.boneMatrices.is_null() &&
            raw.boneCount >= 0 &&
            raw.vaoId != 0 &&
            !raw.vboId.is_null() &&
            // SAFETY: just confirmed non-null
            // TODO: how to ensure number of elements?
            unsafe {
                *raw.vboId != 0 &&
                *raw.vboId.add(1) != 0 &&
                *raw.vboId.add(2) != 0 &&
                *raw.vboId.add(3) != 0 &&
                *raw.vboId.add(4) != 0 &&
                *raw.vboId.add(5) != 0 &&
                *raw.vboId.add(6) != 0
            }
        {
            // SAFETY: Just checked field validity.
            // Responsibiltiy of caller to uphold remaining contracts.
            Some(unsafe { Self::from_raw_unchecked(raw) })
        } else {
            None
        }
    }

    /// Converts raylib-sys object to a "safe" version.
    ///
    /// # Safety
    ///
    /// Mesh must be valid:
    /// - All pointer fields must be non-null, not dangling, and [valid](std::ptr#safety).
    /// - All count fields must be non-negative.
    ///
    /// Make sure to call this function from the thread the resource was created.
    ///
    /// The mesh resource that `raw` represents must be released **exactly once**.
    /// It is strongly recommended to avoid having multiple [`Mesh`]es referring
    /// to the same mesh resource, for the sake of sanity.
    pub const unsafe fn from_raw_unchecked(raw: ffi::Mesh) -> Self {
        // SAFETY: `Mesh` has the same size, fields, and layout as `ffi::Mesh`.
        // Caller must uphold remaining safety contracts.
        unsafe { std::mem::transmute(raw) }
    }
}

// Weak things can be clone
impl Clone for WeakModel {
    fn clone(&self) -> WeakModel {
        WeakModel(self.0)
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

impl RaylibHandle {
    #[must_use]
    /// Loads model from files (mesh and material).
    // #[inline]
    pub fn load_model(
        &mut self,
        _: &RaylibThread,
        filename: &str,
    ) -> Result<Model, LoadModelError> {
        let c_filename = CString::new(filename).unwrap();
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

    #[must_use]
    /// Loads model from a generated mesh
    pub fn load_model_from_mesh(
        &mut self,
        _: &RaylibThread,
        mesh: ffi::Mesh,
    ) -> Result<Model, LoadModelError> {
        let m = unsafe { ffi::LoadModelFromMesh(mesh) };

        if m.meshes.is_null() || m.materials.is_null() {
            return Err(LoadModelError::LoadFromMeshFailed);
        }

        Ok(Model(m))
    }

    #[must_use]
    /// Load model animations from file
    pub fn load_model_animations(
        &mut self,
        _: &RaylibThread,
        filename: &str,
    ) -> Result<Vec<ModelAnimation>, LoadModelAnimError> {
        let c_filename = CString::new(filename).unwrap();
        let mut m_size = 0;
        let m_ptr = unsafe { ffi::LoadModelAnimations(c_filename.as_ptr(), &mut m_size) };
        if m_size <= 0 {
            return Err(LoadModelAnimError::NoAnimationsLoaded {
                path: filename.into(),
            });
        }
        let mut m_vec = Vec::with_capacity(m_size as usize);
        for i in 0..m_size {
            unsafe {
                m_vec.push(ModelAnimation(*m_ptr.offset(i as isize)));
            }
        }
        unsafe {
            ffi::MemFree(m_ptr as *mut ::std::os::raw::c_void);
        }
        Ok(m_vec)
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
    pub unsafe fn make_weak(self) -> WeakModel {
        let m = WeakModel(self.0);
        std::mem::forget(self);
        m
    }
}

pub trait RaylibModel: AsRef<ffi::Model> + AsMut<ffi::Model> {
    #[inline]
    #[must_use]
    /// Local transform matrix
    fn transform(&self) -> &Matrix {
        unsafe { std::mem::transmute(&self.as_ref().transform) }
    }

    #[inline]
    fn set_transform(&mut self, mat: &Matrix) {
        self.as_mut().transform = (*mat).into();
    }

    /// Meshes array
    #[inline]
    #[must_use]
    fn meshes(&self) -> &[ffi::Mesh] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().meshes as *const ffi::Mesh,
                self.as_ref().meshCount as usize,
            )
        }
    }

    // Meshes array
    #[inline]
    #[must_use]
    fn meshes_mut(&mut self) -> &mut [ffi::Mesh] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().meshes as *mut ffi::Mesh,
                self.as_mut().meshCount as usize,
            )
        }
    }
    /// Materials array
    #[inline]
    #[must_use]
    fn materials(&self) -> &[WeakMaterial] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().materials as *const WeakMaterial,
                self.as_ref().materialCount as usize,
            )
        }
    }
    /// Materials array
    #[inline]
    #[must_use]
    fn materials_mut(&mut self) -> &mut [WeakMaterial] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().materials as *mut WeakMaterial,
                self.as_mut().materialCount as usize,
            )
        }
    }
    #[inline]
    #[must_use]
    /// Bones information (skeleton)
    fn bones(&self) -> Option<&[BoneInfo]> {
        if self.as_ref().bones.is_null() {
            return None;
        }

        Some(unsafe {
            std::slice::from_raw_parts(
                self.as_ref().bones as *const BoneInfo,
                self.as_ref().boneCount as usize,
            )
        })
    }
    #[inline]
    #[must_use]
    /// Bones information (skeleton)
    fn bones_mut(&mut self) -> Option<&mut [BoneInfo]> {
        if self.as_ref().bones.is_null() {
            return None;
        }

        Some(unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().bones as *mut BoneInfo,
                self.as_mut().boneCount as usize,
            )
        })
    }
    #[inline]
    #[must_use]
    /// Bones base transformation (pose)
    fn bind_pose(&self) -> Option<&Transform> {
        if self.as_ref().bindPose.is_null() {
            return None;
        }
        Some(unsafe { std::mem::transmute(self.as_ref().bindPose) })
    }
    #[inline]
    #[must_use]
    /// Bones base transformation (pose)
    fn bind_pose_mut(&mut self) -> Option<&mut Transform> {
        if self.as_ref().bindPose.is_null() {
            return None;
        }
        Some(unsafe { std::mem::transmute(self.as_mut().bindPose) })
    }
    #[inline]
    #[must_use]
    /// Check model animation skeleton match
    fn is_model_animation_valid(&self, anim: &ModelAnimation) -> bool {
        unsafe { ffi::IsModelAnimationValid(*self.as_ref(), anim.0) }
    }

    /// Check if a model is ready
    #[inline]
    #[must_use]
    fn is_model_valid(&self) -> bool {
        unsafe { ffi::IsModelValid(*self.as_ref()) }
    }

    /// Compute model bounding box limits (considers all meshes)
    #[inline]
    #[must_use]
    fn get_model_bounding_box(&self) -> BoundingBox {
        unsafe { BoundingBox::from(ffi::GetModelBoundingBox(*self.as_ref())) }
    }
    #[inline]
    /// Set material for a mesh
    fn set_model_mesh_material(
        &mut self,
        mesh_id: i32,
        material_id: i32,
    ) -> Result<(), SetMaterialError> {
        // should this be an assertion?
        if mesh_id >= self.as_ref().meshCount {
            Err(SetMaterialError::MeshIdOutOfBounds)
        } else if material_id >= self.as_ref().materialCount {
            Err(SetMaterialError::MaterialIdOutOfBounds)
        } else {
            unsafe { ffi::SetModelMeshMaterial(self.as_mut(), mesh_id, material_id) };
            Ok(())
        }
    }
}

macro_rules! pointer_field {
    (
        $(#[$shared:meta])*
        $(
            # Safety
            $(#[$shared_safety:meta])*
        )?
        [$($StorageTy:ty)?] -> [$AccessTy:ty]

        $(#[$accessor_meta:meta])*
        $(
            # Safety
            $(#[$accessor_safety:meta])*
        )?
        $field:ident();

        $(#[$mutator_meta:meta])*
        $(
            # Safety
            $(#[$mutator_safety:meta])*
        )?
        $field_mut:ident();
    ) => {
        $(#[$accessor_meta])*
        ///
        $(#[$shared])*
        ///
        /// # Safety
        ///
        /// For the lifetime of the returned slice, the memory it points to must not get mutated.
        ///
        $($(#[$shared_safety])*)?
        ///
        $($(#[$accessor_safety])*)?
        #[inline]
        #[must_use]
        pub const unsafe fn $field(&self) -> &[$AccessTy] {
            $(const _: () = assert!((std::mem::align_of::<$AccessTy>() % std::mem::size_of::<$StorageTy>()) == 0);)?
            debug_assert!(0 <= self.vertex_count && (i32::BITS <= usize::BITS || self.vertex_count <= usize::MAX as i32),
                "vertex_count should always be in usize range");

            let ptr = NonNull::slice_from_raw_parts(
                self.$field.cast::<$AccessTy>(),
                self.vertex_count as usize,
            );
            // SAFETY: All pointer fields of `Mesh` are non-null and assumed to be
            // aligned, dereferenceable, and pointing to valid data.
            // It is the caller's responsibility to enforce Rust's aliasing rules.
            unsafe { ptr.as_ref() }
        }

        $(#[$mutator_meta])*
        ///
        $(#[$shared])*
        ///
        /// # Safety
        ///
        /// For the lifetime of the returned slice, the memory it points to must not get accessed
        /// (read or written) through any other pointer or reference not derived from it.
        ///
        $($(#[$shared_safety])*)?
        ///
        $($(#[$mutator_safety])*)?
        #[inline]
        #[must_use]
        pub const unsafe fn $field_mut(&mut self) -> &mut [$AccessTy] {
            $(const _: () = assert!((std::mem::align_of::<$AccessTy>() % std::mem::size_of::<$StorageTy>()) == 0);)?
            debug_assert!(0 <= self.vertex_count && (i32::BITS <= usize::BITS || self.vertex_count <= usize::MAX as i32),
                "vertex_count should always be in usize range");

            let mut ptr = NonNull::slice_from_raw_parts(
                self.$field.cast::<$AccessTy>(),
                self.vertex_count as usize,
            );
            // SAFETY: All pointer fields of `Mesh` are non-null and assumed to be
            // aligned, dereferenceable, and pointing to valid data.
            // It is the caller's responsibility to enforce Rust's aliasing rules.
            unsafe { ptr.as_mut() }
        }
    };
}

impl Mesh {
    // Mesh is required to already be uploaded in order to be valid.
    //
    // /// Upload mesh vertex data in GPU and provide VAO/VBO ids
    // #[inline]
    // pub unsafe fn upload(&mut self, dynamic: bool) {
    //     unsafe { ffi::UploadMesh(self.as_mut(), dynamic) };
    // }

    /// Update mesh vertex data in GPU for a specific buffer index
    ///
    /// Requires OpenGL 3.3 or ES 2.0 to have any effect
    ///
    /// # Safety
    ///
    /// `data` and `offset` must define a range entirely contained by the
    /// `index`th buffer of `self`.
    #[inline]
    pub unsafe fn update_buffer(&mut self, index: i32, data: &[u8], offset: i32) {
        // SAFETY: `raw` will not be used after the resource is released
        // because it goes out of scope when this method returns, and `self`
        // is not dropped in this method.
        let raw = unsafe { self.make_raw() };

        // SAFETY:
        // - `raw` refers to a loaded Mesh
        // - `dataSize` and `offset` are provided in bytes
        // - Caller must ensure `dataSize` and `offset` define a range
        //   lying entirely within the buffer object's data store
        unsafe {
            ffi::UpdateMeshBuffer(
                raw,
                index,
                data.as_ptr().cast::<c_void>(),
                data.len() as i32,
                offset,
            )
        };
    }

    pointer_field!{
        /// Vertex position (XYZ - 3 components per vertex) (shader-location = 0)
        [f32] -> [Vector3]
        vertices();
        vertices_mut();
    }
    pointer_field!{
        /// Vertex normals (XYZ - 3 components per vertex) (shader-location = 2)
        [f32] -> [Vector3]
        normals();
        normals_mut();
    }
    pointer_field!{
        /// Vertex tangents (XYZW - 4 components per vertex) (shader-location = 4)
        [f32] -> [Vector4]
        tangents();
        tangents_mut();
    }
    pointer_field!{
        /// Vertex colors (RGBA - 4 components per vertex) (shader-location = 3)
        [u8] -> [Color]
        colors();
        colors_mut();
    }
    pointer_field!{
        /// Vertex indices (in case vertex data comes indexed)
        [] -> [u16]
        indices();
        indices_mut();
    }

    /// Generate polygonal mesh
    #[inline]
    #[must_use]
    pub fn gen_mesh_poly(_: &RaylibThread, sides: i32, radius: f32) -> Option<Mesh> {
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshPoly(sides, radius)) }
    }

    /// Generates plane mesh (with subdivisions).
    #[inline]
    #[must_use]
    pub fn gen_mesh_plane(_: &RaylibThread, width: f32, length: f32, res_x: i32, res_z: i32) -> Option<Mesh> {
        assert!(res_x >= 0);
        assert!(res_z >= 0);
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshPlane(width, length, res_x, res_z)) }
    }

    /// Generates cuboid mesh.
    #[inline]
    #[must_use]
    pub fn gen_mesh_cube(_: &RaylibThread, width: f32, height: f32, length: f32) -> Option<Mesh> {
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshCube(width, height, length)) }
    }

    /// Generates sphere mesh (standard sphere).
    #[inline]
    #[must_use]
    pub fn gen_mesh_sphere(_: &RaylibThread, radius: f32, rings: i32, slices: i32) -> Option<Mesh> {
        assert!(rings >= 0);
        assert!(slices >= 0);
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshSphere(radius, rings, slices)) }
    }

    /// Generates half-sphere mesh (no bottom cap).
    #[inline]
    #[must_use]
    pub fn gen_mesh_hemisphere(_: &RaylibThread, radius: f32, rings: i32, slices: i32) -> Option<Mesh> {
        assert!(rings >= 0);
        assert!(slices >= 0);
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshHemiSphere(radius, rings, slices)) }
    }

    /// Generates cylinder mesh.
    #[inline]
    #[must_use]
    pub fn gen_mesh_cylinder(_: &RaylibThread, radius: f32, height: f32, slices: i32) -> Option<Mesh> {
        assert!(slices >= 0);
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshCylinder(radius, height, slices)) }
    }

    /// Generates torus mesh.
    #[inline]
    #[must_use]
    pub fn gen_mesh_torus(_: &RaylibThread, radius: f32, size: f32, rad_seg: i32, sides: i32) -> Option<Mesh> {
        assert!(rad_seg >= 0);
        assert!(sides >= 0);
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshTorus(radius, size, rad_seg, sides)) }
    }

    /// Generates trefoil knot mesh.
    #[inline]
    #[must_use]
    pub fn gen_mesh_knot(_: &RaylibThread, radius: f32, size: f32, rad_seg: i32, sides: i32) -> Option<Mesh> {
        assert!(rad_seg >= 0);
        assert!(sides >= 0);
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshKnot(radius, size, rad_seg, sides)) }
    }

    /// Generates heightmap mesh from image data.
    #[inline]
    #[must_use]
    pub fn gen_mesh_heightmap(_: &RaylibThread, heightmap: &Image, size: impl Into<MintVec3>) -> Option<Mesh> {
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshHeightmap(heightmap.0, size.into())) }
    }

    /// Generates cubes-based map mesh from image data.
    #[inline]
    #[must_use]
    pub fn gen_mesh_cubicmap(
        _: &RaylibThread,
        cubicmap: &Image,
        cube_size: impl Into<MintVec3>,
    ) -> Option<Mesh> {
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshCubicmap(cubicmap.0, cube_size.into())) }
    }

    /// Generate cone/pyramid mesh
    #[inline]
    #[must_use]
    pub fn gen_mesh_cone(_: &RaylibThread, radius: f32, height: f32, slices: i32) -> Option<Mesh> {
        assert!(slices >= 0);
        // SAFETY:
        // - `RaylibThread` proves we are on the correct thread
        // - We are creating a new mesh, so it isn't pointing to an existing resource
        unsafe { Mesh::from_raw(ffi::GenMeshCone(radius, height, slices)) }
    }

    /// Computes mesh bounding box limits.
    // NOTE: minVertex and maxVertex should be transformed by model transform matrix
    #[inline]
    #[must_use]
    pub fn get_mesh_bounding_box(&self) -> BoundingBox {
        // SAFETY: `raw` will not be used after the resource is released
        // because it goes out of scope when this method returns, and `self`
        // is not dropped in this method.
        let raw = unsafe { self.make_raw() };
        // SAFETY: GetMeshBoundingBox has no preconditions.
        unsafe { ffi::GetMeshBoundingBox(raw).into() }
    }

    /// Computes mesh tangents.
    // NOTE: New VBO for tangents is generated at default location and also binded to mesh VAO
    #[inline]
    pub fn gen_mesh_tangents(&mut self, _: &RaylibThread) {
        unsafe {
            ffi::GenMeshTangents(self.as_raw_mut());
        }
    }

    /// Exports mesh as an OBJ file.
    #[inline]
    pub fn export(&self, filename: &str) {
        // SAFETY: `raw` will not be used after the resource is released
        // because it goes out of scope when this method returns, and `self`
        // is not dropped in this method.
        let raw = unsafe { self.make_raw() };
        let c_filename = CString::new(filename).unwrap();
        unsafe {
            ffi::ExportMesh(raw, c_filename.as_ptr());
        }
    }

    /// Export mesh as code file (.h) defining multiple arrays of vertex attributes
    #[inline]
    pub fn export_as_code(&self, filename: &str) {
        // SAFETY: `raw` will not be used after the resource is released
        // because it goes out of scope when this method returns, and `self`
        // is not dropped in this method.
        let raw = unsafe { self.make_raw() };
        let c_filename = CString::new(filename).unwrap();
        unsafe {
            ffi::ExportMeshAsCode(raw, c_filename.as_ptr());
        }
    }
}

impl Material {
    #[must_use]
    #[inline]
    pub unsafe fn make_weak(self) -> WeakMaterial {
        let m = WeakMaterial(self.0);
        std::mem::forget(self);
        m
    }

    /// Load materials from model file
    #[must_use]
    pub fn load_materials(filename: &str) -> Result<Vec<Material>, LoadMaterialError> {
        let c_filename = CString::new(filename).unwrap();
        let mut m_size = 0;
        let m_ptr = unsafe { ffi::LoadMaterials(c_filename.as_ptr(), &mut m_size) };
        if m_size <= 0 {
            return Err(LoadMaterialError::NoneLoaded {
                path: filename.into(),
            });
        }
        let mut m_vec = Vec::with_capacity(m_size as usize);
        for i in 0..m_size {
            unsafe {
                m_vec.push(Material(*m_ptr.offset(i as isize)));
            }
        }
        unsafe {
            ffi::MemFree(m_ptr as *mut ::std::os::raw::c_void);
        }
        Ok(m_vec)
    }
}

impl RaylibMaterial for WeakMaterial {}
impl RaylibMaterial for Material {}

pub trait RaylibMaterial: AsRef<ffi::Material> + AsMut<ffi::Material> {
    /// Material shader
    #[must_use]
    #[inline]
    fn shader(&self) -> &crate::shaders::WeakShader {
        unsafe { std::mem::transmute(&self.as_ref().shader) }
    }
    #[must_use]
    #[inline]
    /// Material shader
    fn shader_mut(&mut self) -> &mut crate::shaders::WeakShader {
        unsafe { std::mem::transmute(&mut self.as_mut().shader) }
    }
    #[must_use]
    #[inline]
    /// Material maps array (MAX_MATERIAL_MAPS)
    fn maps(&self) -> &[MaterialMap] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().maps as *const MaterialMap,
                consts::MAX_MATERIAL_MAPS as usize,
            )
        }
    }
    #[must_use]
    #[inline]
    /// Material maps array (MAX_MATERIAL_MAPS)
    fn maps_mut(&mut self) -> &mut [MaterialMap] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().maps as *mut MaterialMap,
                consts::MAX_MATERIAL_MAPS as usize,
            )
        }
    }

    /// Set texture for a material map type (MATERIAL_MAP_DIFFUSE, MATERIAL_MAP_SPECULAR...)
    #[inline]
    fn set_material_texture(
        &mut self,
        map_type: crate::consts::MaterialMapIndex,
        texture: impl AsRef<ffi::Texture2D>,
    ) {
        unsafe {
            ffi::SetMaterialTexture(self.as_mut(), (map_type as u32) as i32, *texture.as_ref())
        }
    }

    /// Check if a material is valid (shader assigned, map textures loaded in GPU)
    #[inline]
    #[must_use]
    fn is_material_valid(&mut self) -> bool {
        unsafe { ffi::IsMaterialValid(*self.as_ref()) }
    }
}

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
    fn func(tf: &Option<&'a [Transform]>, bone_count: usize) -> &'a [Transform] {
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
impl<'a> DoubleEndedIterator for FramePoseIter<'a> {
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
impl<'a> ExactSizeIterator for FramePoseIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}
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
    fn func(tf: &mut Option<&'a mut [Transform]>, bone_count: usize) -> &'a mut [Transform] {
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
impl<'a> DoubleEndedIterator for FramePoseIterMut<'a> {
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
impl<'a> ExactSizeIterator for FramePoseIterMut<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl RaylibModelAnimation for ModelAnimation {}
impl RaylibModelAnimation for WeakModelAnimation {}

impl ModelAnimation {
    #[inline]
    #[must_use]
    pub unsafe fn make_weak(self) -> WeakModelAnimation {
        let m = WeakModelAnimation(self.0);
        std::mem::forget(self);
        m
    }
}

pub trait RaylibModelAnimation: AsRef<ffi::ModelAnimation> + AsMut<ffi::ModelAnimation> {
    /// Bones information (skeleton)
    #[inline]
    #[must_use]
    fn bones(&self) -> &[BoneInfo] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().bones as *const BoneInfo,
                self.as_ref().boneCount as usize,
            )
        }
    }

    /// Bones information (skeleton)
    #[inline]
    #[must_use]
    fn bones_mut(&mut self) -> &mut [BoneInfo] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().bones as *mut BoneInfo,
                self.as_mut().boneCount as usize,
            )
        }
    }

    #[must_use]
    /// Poses array by frame
    fn frame_poses(&self) -> Vec<&[Transform]> {
        let anim = self.as_ref();
        let mut top = Vec::with_capacity(anim.frameCount as usize);

        for i in 0..anim.frameCount {
            top.push(unsafe {
                std::slice::from_raw_parts(
                    *(anim.framePoses.offset(i as isize) as *const *const Transform),
                    anim.boneCount as usize,
                )
            });
        }

        top
    }
    #[must_use]
    fn frame_poses_iter<'a>(&'a self) -> FramePoseIter<'a> {
        let anim = self.as_ref();
        unsafe {
            FramePoseIter::new(
                anim.framePoses,
                anim.frameCount as usize,
                anim.boneCount as usize,
            )
        }
    }

    #[must_use]
    /// Poses array by frame
    fn frame_poses_mut(&mut self) -> Vec<&mut [Transform]> {
        let anim = self.as_ref();
        let mut top = Vec::with_capacity(anim.frameCount as usize);

        for i in 0..anim.frameCount {
            top.push(unsafe {
                std::slice::from_raw_parts_mut(
                    *(anim.framePoses.offset(i as isize) as *mut *mut Transform),
                    anim.boneCount as usize,
                )
            });
        }

        top
    }
    #[must_use]
    fn frame_poses_iter_mut<'a>(&'a mut self) -> FramePoseIterMut<'a> {
        let anim = self.as_ref();
        unsafe {
            FramePoseIterMut::new(
                anim.framePoses,
                anim.frameCount as usize,
                anim.boneCount as usize,
            )
        }
    }
}

impl MaterialMap {
    /// Material map texture
    #[inline]
    #[must_use]
    pub fn texture(&self) -> &crate::texture::WeakTexture2D {
        unsafe { std::mem::transmute(&self.0.texture) }
    }
    /// Material map texture
    #[inline]
    #[must_use]
    pub fn texture_mut(&mut self) -> &mut crate::texture::WeakTexture2D {
        unsafe { std::mem::transmute(&mut self.0.texture) }
    }

    /// Material map color
    #[inline]
    #[must_use]
    pub fn color(&self) -> &Color {
        unsafe { std::mem::transmute(&self.0.color) }
    }
    /// Material map color
    #[inline]
    #[must_use]
    pub fn color_mut(&mut self) -> &mut Color {
        unsafe { std::mem::transmute(&mut self.0.color) }
    }

    /// Material map value
    #[inline]
    #[must_use]
    pub fn value(&self) -> &f32 {
        unsafe { std::mem::transmute(&self.0.value) }
    }
    /// Material map value
    #[inline]
    #[must_use]
    pub fn value_mut(&mut self) -> &mut f32 {
        unsafe { std::mem::transmute(&mut self.0.value) }
    }
}

impl RaylibHandle {
    /// Load default material (Supports: DIFFUSE, SPECULAR, NORMAL maps)
    #[inline]
    #[must_use]
    pub fn load_material_default(&self, _: &RaylibThread) -> WeakMaterial {
        WeakMaterial(unsafe { ffi::LoadMaterialDefault() })
    }

    /// Weak materials will leak memeory if they are not unlaoded
    /// Unload material from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_material(&mut self, _: &RaylibThread, material: WeakMaterial) {
        unsafe { ffi::UnloadMaterial(*material.as_ref()) }
    }

    /// Weak models will leak memeory if they are not unlaoded
    /// Unload model from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_model(&mut self, _: &RaylibThread, model: WeakModel) {
        unsafe { ffi::UnloadModel(*model.as_ref()) }
    }

    /// Weak model_animations will leak memeory if they are not unlaoded
    /// Unload model_animation from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_model_animation(
        &mut self,
        _: &RaylibThread,
        model_animation: WeakModelAnimation,
    ) {
        unsafe { ffi::UnloadModelAnimation(*model_animation.as_ref()) }
    }

    /// Weak meshs will leak memeory if they are not unlaoded
    /// Unload mesh from GPU memory (VRAM)
    ///
    /// # Safety
    ///
    /// Any unique mesh must be unloaded **exactly once**.
    /// The drop implementation on [`Mesh`] already does this,
    /// you do not need to convert your [`Mesh`] into a weak
    /// [`ffi::Mesh`] in order to unload it.
    #[inline]
    pub unsafe fn unload_mesh(&mut self, _: &RaylibThread, mesh: ffi::Mesh) {
        unsafe { ffi::UnloadMesh(mesh) }
    }
}
