//! 3D Model, Mesh, and Animation

use crate::core::databuf::DataBuf;
use crate::core::math::BoundingBox;
use crate::core::math::Matrix;
use crate::core::math::Transform;
use crate::core::math::{Vector2, Vector3, Vector4};
use crate::core::texture::Image;
use crate::core::{RaylibHandle, RaylibThread};
use crate::ffi::Color;
use crate::{
    consts,
    error::{
        AllocationError, GenMeshError, InvalidMeshError, LoadMaterialError, LoadModelAnimError,
        LoadModelError, SetMaterialError,
    },
    ffi,
};
use std::ffi::CString;
use std::os::raw::c_void;

fn no_drop<T>(_thing: T) {}
make_thin_wrapper!(
    /// Model, meshes, materials and animation data
    Model,
    ffi::Model,
    ffi::UnloadModel
);
make_thin_wrapper!(WeakModel, ffi::Model, no_drop);
make_thin_wrapper!(
    /// Mesh, vertex data and vao/vbo
    Mesh,
    ffi::Mesh,
    |mesh: ffi::Mesh| ffi::UnloadMesh(mesh)
);
make_thin_wrapper!(WeakMesh, ffi::Mesh, no_drop);
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
    /// A single model animation: a non-owning view into a `ModelAnimations` collection.
    ModelAnimation,
    ffi::ModelAnimation,
    no_drop
);
make_thin_wrapper!(WeakModelAnimation, ffi::ModelAnimation, no_drop);
make_thin_wrapper!(
    /// MaterialMap
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

/// Owns the heap array returned by `LoadModelAnimations`. Frees it exactly once on
/// drop via `UnloadModelAnimations` (which frees each animation's keyframe poses AND
/// the array pointer). Individual animations are borrowed, non-owning `ModelAnimation`s.
#[derive(Debug)]
pub struct ModelAnimations {
    ptr: *mut ffi::ModelAnimation,
    count: usize,
}

impl ModelAnimations {
    /// Number of animations in the array.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.count
    }
    /// Whether the array is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }
    /// The animations as a borrowed slice of non-owning views.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[ModelAnimation] {
        // SAFETY: ModelAnimation is #[repr(transparent)] over ffi::ModelAnimation and the
        // array of `count` elements at `ptr` is valid for the lifetime of `self`.
        unsafe { std::slice::from_raw_parts(self.ptr as *const ModelAnimation, self.count) }
    }
    /// The animations as a mutable borrowed slice of non-owning views.
    #[inline]
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [ModelAnimation] {
        // SAFETY: see as_slice; exclusive borrow guarantees no aliasing.
        unsafe { std::slice::from_raw_parts_mut(self.ptr as *mut ModelAnimation, self.count) }
    }
}

impl std::ops::Deref for ModelAnimations {
    type Target = [ModelAnimation];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl std::ops::DerefMut for ModelAnimations {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl Drop for ModelAnimations {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: `ptr`/`count` are exactly what LoadModelAnimations returned; 6.0's
            // UnloadModelAnimations frees each keyframePoses[i], keyframePoses, then the
            // array. Called exactly once (this owner is the sole holder of `ptr`).
            unsafe { ffi::UnloadModelAnimations(self.ptr, self.count as i32) }
        }
    }
}

impl RaylibHandle {
    /// Loads model from files (mesh and material).
    // #[inline]
    pub fn load_model(
        &mut self,
        _: &RaylibThread,
        filename: &str,
    ) -> Result<Model, LoadModelError> {
        let c_filename = CString::new(filename).unwrap();
        let m = unsafe { ffi::LoadModel(c_filename.as_ptr()) };
        if m.meshes.is_null()
            && m.materials.is_null()
            && m.skeleton.bones.is_null()
            && m.skeleton.bindPose.is_null()
        {
            return Err(LoadModelError::LoadFromFileFailed {
                path: filename.into(),
            });
        }
        // TODO check if null pointer checks are necessary.
        Ok(Model(m))
    }

    /// Loads model from a generated mesh
    pub fn load_model_from_mesh(
        &mut self,
        _: &RaylibThread,
        mesh: WeakMesh,
    ) -> Result<Model, LoadModelError> {
        let m = unsafe { ffi::LoadModelFromMesh(mesh.0) };

        if m.meshes.is_null() || m.materials.is_null() {
            return Err(LoadModelError::LoadFromMeshFailed);
        }

        Ok(Model(m))
    }

    /// Load model animations from file
    pub fn load_model_animations(
        &mut self,
        _: &RaylibThread,
        filename: &str,
    ) -> Result<ModelAnimations, LoadModelAnimError> {
        let c_filename = CString::new(filename).unwrap();
        let mut m_size = 0;
        let m_ptr = unsafe { ffi::LoadModelAnimations(c_filename.as_ptr(), &mut m_size) };
        if m_ptr.is_null() || m_size <= 0 {
            return Err(LoadModelAnimError::NoAnimationsLoaded {
                path: filename.into(),
            });
        }
        Ok(ModelAnimations {
            ptr: m_ptr,
            count: m_size as usize,
        })
    }

    /// Update model animation pose (CPU)
    #[inline]
    pub fn update_model_animation(
        &mut self,
        _: &RaylibThread,
        mut model: impl AsMut<ffi::Model>,
        anim: impl AsRef<ffi::ModelAnimation>,
        frame: f32,
    ) {
        unsafe {
            ffi::UpdateModelAnimation(*model.as_mut(), *anim.as_ref(), frame);
        }
    }

    /// Update model animation pose, blending two animations (CPU).
    ///
    /// New in raylib 6.0. `blend` (0.0..=1.0) interpolates between the pose of
    /// `anim_a` at `frame_a` and `anim_b` at `frame_b`.
    #[inline]
    #[allow(clippy::too_many_arguments)] // mirrors the raylib C API exactly; no natural grouping
    pub fn update_model_animation_ex(
        &mut self,
        _: &RaylibThread,
        mut model: impl AsMut<ffi::Model>,
        anim_a: impl AsRef<ffi::ModelAnimation>,
        frame_a: f32,
        anim_b: impl AsRef<ffi::ModelAnimation>,
        frame_b: f32,
        blend: f32,
    ) {
        unsafe {
            ffi::UpdateModelAnimationEx(
                *model.as_mut(),
                *anim_a.as_ref(),
                frame_a,
                *anim_b.as_ref(),
                frame_b,
                blend,
            );
        }
    }
}

impl RaylibModel for WeakModel {}
impl RaylibModel for Model {}

impl Model {
    /// # Safety
    ///
    /// The caller becomes responsible for ensuring the underlying `ffi::Model` is eventually
    /// unloaded. The returned `WeakModel` does not call `UnloadModel` on drop.
    pub unsafe fn make_weak(self) -> WeakModel {
        let m = WeakModel(self.0);
        std::mem::forget(self);
        m
    }
}

/// Methods for querying and mutating a loaded 3D model's meshes, materials, bones, and transform.
pub trait RaylibModel: AsRef<ffi::Model> + AsMut<ffi::Model> {
    #[inline]
    #[must_use]
    /// Local transform matrix
    fn transform(&self) -> &Matrix {
        unsafe { std::mem::transmute(&self.as_ref().transform) }
    }

    /// Sets the model's local transform matrix.
    #[inline]
    fn set_transform(&mut self, mat: &Matrix) {
        self.as_mut().transform = *mat;
    }

    /// Meshes array
    #[inline]
    #[must_use]
    fn meshes(&self) -> &[WeakMesh] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().meshes as *const WeakMesh,
                self.as_ref().meshCount as usize,
            )
        }
    }
    /// Meshes array (mutable)
    #[inline]
    #[must_use]
    fn meshes_mut(&mut self) -> &mut [WeakMesh] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().meshes as *mut WeakMesh,
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
        if self.as_ref().skeleton.bones.is_null() {
            return None;
        }

        Some(unsafe {
            std::slice::from_raw_parts(
                self.as_ref().skeleton.bones as *const BoneInfo,
                self.as_ref().skeleton.boneCount as usize,
            )
        })
    }
    #[inline]
    #[must_use]
    /// Bones information (skeleton)
    fn bones_mut(&mut self) -> Option<&mut [BoneInfo]> {
        if self.as_ref().skeleton.bones.is_null() {
            return None;
        }

        Some(unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().skeleton.bones as *mut BoneInfo,
                self.as_mut().skeleton.boneCount as usize,
            )
        })
    }
    #[inline]
    #[must_use]
    /// Bones base transformation (pose)
    fn bind_pose(&self) -> Option<&Transform> {
        if self.as_ref().skeleton.bindPose.is_null() {
            return None;
        }
        // SAFETY: bindPose is non-null (checked above) and points to a valid ffi::Transform.
        // Transform is #[repr(C)] over ffi::Transform, so the cast is sound.
        Some(unsafe { &*(self.as_ref().skeleton.bindPose as *const Transform) })
    }
    #[inline]
    #[must_use]
    /// Bones base transformation (pose)
    fn bind_pose_mut(&mut self) -> Option<&mut Transform> {
        if self.as_ref().skeleton.bindPose.is_null() {
            return None;
        }
        // SAFETY: bindPose is non-null (checked above) and points to a valid ffi::Transform.
        // Transform is #[repr(C)] over ffi::Transform, so the cast is sound.
        Some(unsafe { &mut *(self.as_mut().skeleton.bindPose as *mut Transform) })
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

impl RaylibMesh for WeakMesh {}
impl RaylibMesh for Mesh {}

impl Mesh {
    /// # Safety
    ///
    /// The caller becomes responsible for ensuring the underlying `ffi::Mesh` is eventually
    /// unloaded. The returned `WeakMesh` does not call `UnloadMesh` on drop.
    pub unsafe fn make_weak(self) -> WeakMesh {
        let m = WeakMesh(self.0);
        std::mem::forget(self);
        m
    }
}
/// Methods for querying and mutating mesh vertex data, uploading to GPU, and generating mesh shapes.
pub trait RaylibMesh: AsRef<ffi::Mesh> + AsMut<ffi::Mesh> {
    /// Upload mesh vertex data in GPU and provide VAO/VBO ids
    ///
    /// # Safety
    ///
    /// The mesh must have valid vertex data (vertices and texcoords at minimum). The mesh
    /// must not already have GPU buffers allocated (i.e., `vaoId` and all `vboId` must be 0).
    #[inline]
    unsafe fn upload(&mut self, dynamic: bool) {
        unsafe { ffi::UploadMesh(self.as_mut(), dynamic) };
    }
    /// Update mesh vertex data in GPU for a specific buffer index
    ///
    /// # Safety
    ///
    /// `index` must be a valid VBO index for this mesh (0..=6). The mesh must already be
    /// uploaded to the GPU. `data` must be a valid byte slice for the buffer at `index`.
    #[inline]
    unsafe fn update_buffer<A>(&mut self, index: i32, data: &[u8], offset: i32) {
        unsafe {
            ffi::UpdateMeshBuffer(
                *self.as_ref(),
                index,
                data.as_ptr() as *const c_void,
                data.len() as i32,
                offset,
            )
        };
    }
    /// Vertex position (XYZ - 3 components per vertex) (shader-location = 0)
    #[inline]
    #[must_use]
    fn vertices(&self) -> &[Vector3] {
        let m = self.as_ref();
        if m.vertices.is_null() || m.vertexCount == 0 {
            return &[];
        }
        // SAFETY: vertices is non-null, points to vertexCount * Vector3 (3 × f32),
        // raylib-allocated and valid for the lifetime of this borrow.
        unsafe { std::slice::from_raw_parts(m.vertices as *const Vector3, m.vertexCount as usize) }
    }
    /// Vertex position (XYZ - 3 components per vertex) (shader-location = 0)
    #[inline]
    #[must_use]
    fn vertices_mut(&mut self) -> &mut [Vector3] {
        let m = self.as_mut();
        if m.vertices.is_null() || m.vertexCount == 0 {
            return &mut [];
        }
        // SAFETY: vertices is non-null, exclusively borrowed, valid for vertexCount elements.
        unsafe {
            std::slice::from_raw_parts_mut(m.vertices as *mut Vector3, m.vertexCount as usize)
        }
    }
    /// Vertex normals (XYZ - 3 components per vertex) (shader-location = 2)
    #[inline]
    #[must_use]
    fn normals(&self) -> &[Vector3] {
        let m = self.as_ref();
        if m.normals.is_null() || m.vertexCount == 0 {
            return &[];
        }
        // SAFETY: normals is non-null, points to vertexCount * Vector3, valid for this borrow.
        unsafe { std::slice::from_raw_parts(m.normals as *const Vector3, m.vertexCount as usize) }
    }
    /// Vertex normals (XYZ - 3 components per vertex) (shader-location = 2)
    #[inline]
    #[must_use]
    fn normals_mut(&mut self) -> &mut [Vector3] {
        let m = self.as_mut();
        if m.normals.is_null() || m.vertexCount == 0 {
            return &mut [];
        }
        // SAFETY: normals is non-null, exclusively borrowed, valid for vertexCount elements.
        unsafe { std::slice::from_raw_parts_mut(m.normals as *mut Vector3, m.vertexCount as usize) }
    }
    /// Vertex texture coordinates (UV - 2 components per vertex) (shader-location = 1)
    #[inline]
    #[must_use]
    fn texcoords(&self) -> &[Vector2] {
        let m = self.as_ref();
        if m.texcoords.is_null() || m.vertexCount == 0 {
            return &[];
        }
        // SAFETY: texcoords is non-null, points to vertexCount * Vector2 (2 × f32),
        // raylib-allocated and valid for this borrow's lifetime.
        unsafe { std::slice::from_raw_parts(m.texcoords as *const Vector2, m.vertexCount as usize) }
    }
    /// Vertex texture coordinates (UV - 2 components per vertex) (shader-location = 1)
    #[inline]
    #[must_use]
    fn texcoords_mut(&mut self) -> &mut [Vector2] {
        let m = self.as_mut();
        if m.texcoords.is_null() || m.vertexCount == 0 {
            return &mut [];
        }
        // SAFETY: texcoords is non-null, exclusively borrowed, valid for vertexCount elements.
        unsafe {
            std::slice::from_raw_parts_mut(m.texcoords as *mut Vector2, m.vertexCount as usize)
        }
    }
    /// Vertex texture second coordinates (UV - 2 components per vertex) (shader-location = 5)
    #[inline]
    #[must_use]
    fn texcoords2(&self) -> &[Vector2] {
        let m = self.as_ref();
        if m.texcoords2.is_null() || m.vertexCount == 0 {
            return &[];
        }
        // SAFETY: texcoords2 is non-null, points to vertexCount * Vector2, valid for this borrow.
        unsafe {
            std::slice::from_raw_parts(m.texcoords2 as *const Vector2, m.vertexCount as usize)
        }
    }
    /// Vertex texture second coordinates (UV - 2 components per vertex) (shader-location = 5)
    #[inline]
    #[must_use]
    fn texcoords2_mut(&mut self) -> &mut [Vector2] {
        let m = self.as_mut();
        if m.texcoords2.is_null() || m.vertexCount == 0 {
            return &mut [];
        }
        // SAFETY: texcoords2 is non-null, exclusively borrowed, valid for vertexCount elements.
        unsafe {
            std::slice::from_raw_parts_mut(m.texcoords2 as *mut Vector2, m.vertexCount as usize)
        }
    }
    /// Vertex tangents (XYZW - 4 components per vertex) (shader-location = 4)
    #[inline]
    #[must_use]
    fn tangents(&self) -> &[Vector3] {
        let m = self.as_ref();
        if m.tangents.is_null() || m.vertexCount == 0 {
            return &[];
        }
        // SAFETY: tangents is non-null, points to vertexCount * Vector3, valid for this borrow.
        unsafe { std::slice::from_raw_parts(m.tangents as *const Vector3, m.vertexCount as usize) }
    }
    /// Vertex tangents (XYZW - 4 components per vertex) (shader-location = 4)
    #[inline]
    #[must_use]
    fn tangents_mut(&mut self) -> &mut [Vector3] {
        let m = self.as_mut();
        if m.tangents.is_null() || m.vertexCount == 0 {
            return &mut [];
        }
        // SAFETY: tangents is non-null, exclusively borrowed, valid for vertexCount elements.
        unsafe {
            std::slice::from_raw_parts_mut(m.tangents as *mut Vector3, m.vertexCount as usize)
        }
    }
    /// Vertex colors (RGBA - 4 components per vertex) (shader-location = 3)
    #[inline]
    #[must_use]
    fn colors(&self) -> &[Color] {
        let m = self.as_ref();
        if m.colors.is_null() || m.vertexCount == 0 {
            return &[];
        }
        // SAFETY: colors is non-null, points to vertexCount * Color (4 × u8), valid for this borrow.
        unsafe { std::slice::from_raw_parts(m.colors as *const Color, m.vertexCount as usize) }
    }
    /// Vertex colors (RGBA - 4 components per vertex) (shader-location = 3)
    #[inline]
    #[must_use]
    fn colors_mut(&mut self) -> &mut [Color] {
        let m = self.as_mut();
        if m.colors.is_null() || m.vertexCount == 0 {
            return &mut [];
        }
        // SAFETY: colors is non-null, exclusively borrowed, valid for vertexCount elements.
        unsafe { std::slice::from_raw_parts_mut(m.colors as *mut Color, m.vertexCount as usize) }
    }
    /// Vertex indices (in case vertex data comes indexed) — triangleCount * 3 entries
    #[inline]
    #[must_use]
    fn indices(&self) -> &[u16] {
        let m = self.as_ref();
        if m.indices.is_null() || m.triangleCount == 0 {
            return &[];
        }
        // SAFETY: indices is non-null, points to triangleCount * 3 u16 entries,
        // raylib-allocated and valid for this borrow's lifetime.
        unsafe { std::slice::from_raw_parts(m.indices as *const u16, m.triangleCount as usize * 3) }
    }
    /// Vertex indices (in case vertex data comes indexed) — triangleCount * 3 entries
    #[inline]
    #[must_use]
    fn indices_mut(&mut self) -> &mut [u16] {
        let m = self.as_mut();
        if m.indices.is_null() || m.triangleCount == 0 {
            return &mut [];
        }
        // SAFETY: indices is non-null, exclusively borrowed, triangleCount * 3 u16 entries.
        unsafe { std::slice::from_raw_parts_mut(m.indices, m.triangleCount as usize * 3) }
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
    fn gen_mesh_heightmap(_: &RaylibThread, heightmap: &Image, size: impl Into<Vector3>) -> Mesh {
        unsafe { Mesh(ffi::GenMeshHeightmap(heightmap.0, size.into())) }
    }

    /// Generates cubes-based map mesh from image data.
    #[inline]
    #[must_use]
    fn gen_mesh_cubicmap(
        _: &RaylibThread,
        cubicmap: &Image,
        cube_size: impl Into<Vector3>,
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
    fn get_mesh_bounding_box(&self) -> BoundingBox {
        unsafe { ffi::GetMeshBoundingBox(*self.as_ref()).into() }
    }

    /// Computes mesh tangents.
    // NOTE: New VBO for tangents is generated at default location and also binded to mesh VAO
    #[inline]
    fn gen_mesh_tangents(&mut self, _: &RaylibThread) {
        unsafe {
            ffi::GenMeshTangents(self.as_mut());
        }
    }

    /// Exports mesh as an OBJ file.
    #[inline]
    fn export(&self, filename: &str) {
        let c_filename = CString::new(filename).unwrap();
        unsafe {
            ffi::ExportMesh(*self.as_ref(), c_filename.as_ptr());
        }
    }

    /// Export mesh as code file (.h) defining multiple arrays of vertex attributes
    #[inline]
    fn export_as_code(&self, filename: &str) {
        let c_filename = CString::new(filename).unwrap();
        unsafe {
            ffi::ExportMeshAsCode(*self.as_ref(), c_filename.as_ptr());
        }
    }
}

impl Material {
    /// # Safety
    ///
    /// The caller becomes responsible for ensuring the underlying `ffi::Material` is eventually
    /// unloaded. The returned `WeakMaterial` does not call `UnloadMaterial` on drop.
    #[inline]
    pub unsafe fn make_weak(self) -> WeakMaterial {
        let m = WeakMaterial(self.0);
        std::mem::forget(self);
        m
    }

    /// Load materials from model file
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

/// Methods for querying and mutating a material's shader, texture maps, and validity.
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

/// An iterator over the per-frame bone transform slices of a model animation (immutable).
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
impl DoubleEndedIterator for FramePoseIter<'_> {
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
impl ExactSizeIterator for FramePoseIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}
/// An iterator over the per-frame bone transform slices of a model animation (mutable).
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
    /// # Safety
    ///
    /// The caller becomes responsible for ensuring the underlying `ffi::ModelAnimation` is
    /// eventually unloaded. The returned `WeakModelAnimation` does not call any unload fn on drop.
    #[inline]
    #[must_use]
    pub unsafe fn make_weak(self) -> WeakModelAnimation {
        let m = WeakModelAnimation(self.0);
        std::mem::forget(self);
        m
    }
}

/// Methods for accessing keyframe pose data from a model animation.
pub trait RaylibModelAnimation: AsRef<ffi::ModelAnimation> + AsMut<ffi::ModelAnimation> {
    #[must_use]
    /// Poses array by frame
    fn frame_poses(&self) -> Vec<&[Transform]> {
        let anim = self.as_ref();
        let mut top = Vec::with_capacity(anim.keyframeCount as usize);

        for i in 0..anim.keyframeCount {
            top.push(unsafe {
                std::slice::from_raw_parts(
                    *(anim.keyframePoses.offset(i as isize) as *const *const Transform),
                    anim.boneCount as usize,
                )
            });
        }

        top
    }
    /// Returns an iterator over each frame's bone transform slice (immutable).
    #[must_use]
    fn frame_poses_iter(&self) -> FramePoseIter<'_> {
        let anim = self.as_ref();
        unsafe {
            FramePoseIter::new(
                anim.keyframePoses,
                anim.keyframeCount as usize,
                anim.boneCount as usize,
            )
        }
    }

    #[must_use]
    /// Poses array by frame
    fn frame_poses_mut(&mut self) -> Vec<&mut [Transform]> {
        let anim = self.as_ref();
        let mut top = Vec::with_capacity(anim.keyframeCount as usize);

        for i in 0..anim.keyframeCount {
            top.push(unsafe {
                std::slice::from_raw_parts_mut(
                    *(anim.keyframePoses.offset(i as isize) as *mut *mut Transform),
                    anim.boneCount as usize,
                )
            });
        }

        top
    }
    /// Returns an iterator over each frame's bone transform slice (mutable).
    #[must_use]
    fn frame_poses_iter_mut(&mut self) -> FramePoseIterMut<'_> {
        let anim = self.as_ref();
        unsafe {
            FramePoseIterMut::new(
                anim.keyframePoses,
                anim.keyframeCount as usize,
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

    /// Unload material from GPU memory (VRAM)
    ///
    /// # Safety
    ///
    /// `material` must not be used after this call. Weak materials will leak memory if not unloaded.
    #[inline]
    pub unsafe fn unload_material(&mut self, _: &RaylibThread, material: WeakMaterial) {
        unsafe { ffi::UnloadMaterial(*material.as_ref()) }
    }

    /// Unload model from GPU memory (VRAM)
    ///
    /// # Safety
    ///
    /// `model` must not be used after this call. Weak models will leak memory if not unloaded.
    #[inline]
    pub unsafe fn unload_model(&mut self, _: &RaylibThread, model: WeakModel) {
        unsafe { ffi::UnloadModel(*model.as_ref()) }
    }

    /// Unload mesh from GPU memory (VRAM)
    ///
    /// # Safety
    ///
    /// `mesh` must not be used after this call. Weak meshes will leak memory if not unloaded.
    #[inline]
    pub unsafe fn unload_mesh(&mut self, _: &RaylibThread, mesh: WeakMesh) {
        unsafe { ffi::UnloadMesh(*mesh.as_ref()) }
    }
}

/// Builder for constructing a custom [`Mesh`] from vertex data before uploading to the GPU.
#[derive(Debug, Clone)]
#[must_use]
pub struct MeshBuilder<'a> {
    /// Vertex position (XYZ - 3 components per vertex)
    vertices: &'a [Vector3],
    /// Vertex texture coordinates (UV - 2 components per vertex)
    texcoords: &'a [Vector2],
    /// Vertex texture second coordinates (UV - 2 components per vertex)
    texcoords2: Option<&'a [Vector2]>,
    /// Vertex normals (XYZ - 3 components per vertex)
    normals: Option<&'a [Vector3]>,
    /// Vertex tangents (XYZW - 4 components per vertex)
    tangents: Option<&'a [Vector4]>,
    /// Vertex colors (RGBA - 4 components per vertex)
    colors: Option<&'a [Color]>,
    /// Vertex indices (in case vertex data comes indexed)
    indices: Option<&'a [u16]>,
}

impl Mesh {
    /// Create a new [`MeshBuilder`] to begin generating a custom [`Mesh`].
    ///
    /// # Example
    /// ```no_run
    /// # use raylib::prelude::*;
    /// # let (mut rl, thread) = init().build();
    /// let mesh = Mesh::gen_mesh(&[
    ///     Vector3::new(0.0, 0.0, 0.0),
    ///     Vector3::new(1.0, 0.0, 0.0),
    ///     Vector3::new(1.0, 0.0, 1.0),
    /// ], &[
    ///     Vector2::new(0.0, 0.0),
    ///     Vector2::new(1.0, 0.0),
    ///     Vector2::new(1.0, 1.0),
    /// ])
    /// .normals(&[
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    /// ])
    /// .colors(&[
    ///     Color::RED,
    ///     Color::GREEN,
    ///     Color::BLUE,
    /// ])
    /// .build(&thread);
    /// ```
    #[inline]
    pub fn gen_mesh<'a>(vertices: &'a [Vector3], texcoords: &'a [Vector2]) -> MeshBuilder<'a> {
        MeshBuilder::new(vertices, texcoords)
    }
}

/// Allocate a Raylib-managed pointer to a copy of `[T]` cast to `U` for use in [`ffi::Mesh`].
///
/// This function is safe, but dereferencing the returned pointer may not be.
/// The caller must ensure that `*mut [T]` is safe to dereference as `*mut U`.
fn slice_to_rl_ptr<'a, T: Copy + 'a, U: 'a>(
    data: Option<&'a [T]>,
) -> Result<*mut U, AllocationError> {
    Ok(match data {
        Some(data) => {
            // ok:  {AAAA} -> {AAAA}
            // ok:  {AAAA} -> {AA}{AA}
            // bad: {AAAA} -> {AAAA????}
            assert!(
                std::mem::size_of_val(data) >= std::mem::size_of::<U>(),
                "should not cast to a larger type",
            );
            // ok:  {AAAA} -> {AAAA}
            // ok:  {AAAA} -> {AA}{AA}
            // bad: {AAAA} -> {AAA}{A??}
            assert!(
                (std::mem::size_of_val(data) % std::mem::size_of::<U>()) == 0,
                "should not cast to a type whose size does not evenly divide the source",
            );
            // ok:  {AAAA|BBBB} -> {AA|AA|BB|BB}
            // ok:  {AAAA|BBBB} -> {A|A|A|A|B|B|B|B}
            // bad: {AAAA|BBBB} -> {AAAABBBB|????????}
            assert!(
                (std::mem::align_of::<T>() >= std::mem::align_of::<U>()),
                "should not cast to a type with wider alignment than that of the source",
            );
            // ok:  {AAAA|BBBB} -> {AA|AA}{BB|BB}
            // ok:  {AAAA|BBBB} -> {AA}{AA}{BB}{BB}
            // bad: {AAAA|BBBB} -> {AAA|ABB}{BB?|???}
            assert!(
                (std::mem::align_of::<T>() % std::mem::align_of::<U>()) == 0,
                "should not cast to a type whose alignment does not evenly divide the source alignment",
            );
            DataBuf::<[T]>::alloc_from_copy(data)?
                .into_inner()
                .into_inner()
                .as_ptr()
                .cast::<U>()
        }
        // Raylib accepts null for optional pointer values, so it's ok to provide `null_mut`.
        None => std::ptr::null_mut(),
    })
}

impl<'a> MeshBuilder<'a> {
    /// Construct a [`MeshBuilder`] from its required fields.
    ///
    /// NOTE: `texcoords` should have the same number of elements as `vertices`.
    pub fn new(vertices: &'a [Vector3], texcoords: &'a [Vector2]) -> Self {
        Self {
            vertices,
            texcoords,
            texcoords2: None,
            normals: None,
            tangents: None,
            colors: None,
            indices: None,
        }
    }

    /// Give the mesh custom secondary texture coordinates.
    ///
    /// NOTE: `texcoords2` should have the same number of elements as `self.vertices`.
    #[inline]
    pub fn texcoords2(&mut self, texcoords2: &'a [Vector2]) -> &mut Self {
        assert!(
            self.texcoords2.is_none(),
            "texcoords2() should be called no more than once on the same MeshBuilder",
        );
        self.texcoords2 = Some(texcoords2);
        self
    }

    /// Give the mesh custom vertex normals.
    ///
    /// NOTE: `normals` should have the same number of elements as `self.vertices`.
    #[inline]
    pub fn normals(&mut self, normals: &'a [Vector3]) -> &mut Self {
        assert!(
            self.normals.is_none(),
            "normals() should be called no more than once on the same MeshBuilder",
        );
        self.normals = Some(normals);
        self
    }

    /// Give the mesh custom tangent vectors.
    ///
    /// NOTE: `tangents` should have the same number of elements as `self.vertices`.
    #[inline]
    pub fn tangents(&mut self, tangents: &'a [Vector4]) -> &mut Self {
        assert!(
            self.tangents.is_none(),
            "tangents() should be called no more than once on the same MeshBuilder",
        );
        self.tangents = Some(tangents);
        self
    }

    /// Give the mesh custom vertex colors.
    ///
    /// NOTE: `colors` should have the same number of elements as `self.vertices`.
    #[inline]
    pub fn colors(&mut self, colors: &'a [Color]) -> &mut Self {
        assert!(
            self.colors.is_none(),
            "colors() should be called no more than once on the same MeshBuilder",
        );
        self.colors = Some(colors);
        self
    }

    /// Give the mesh custom triangle indices.
    ///
    /// NOTE: `indices` should have 3x as many elements as `self.triangle_count`.
    #[inline]
    pub fn indices(&mut self, indices: &'a [u16]) -> &mut Self {
        assert!(
            self.indices.is_none(),
            "indices() should be called no more than once on the same MeshBuilder",
        );
        self.indices = Some(indices);
        self
    }

    fn check_mesh(&self) -> Result<(usize, usize), InvalidMeshError> {
        let vertex_count = self.vertices.len();
        let triangle_vertex_count = self.indices.map_or(vertex_count, <[_]>::len);
        let triangle_count = triangle_vertex_count / 3;
        let triangle_count_rem = triangle_vertex_count % 3;
        if triangle_count_rem != 0 {
            Err(InvalidMeshError::TrianglePointMiscount)
        } else if self.texcoords.len() != vertex_count {
            Err(InvalidMeshError::TexcoordsMiscount)
        } else if self.texcoords2.is_some_and(|x| x.len() != vertex_count) {
            Err(InvalidMeshError::Texcoords2Miscount)
        } else if self.normals.is_some_and(|x| x.len() != vertex_count) {
            Err(InvalidMeshError::NormalsMiscount)
        } else if self.tangents.is_some_and(|x| x.len() != vertex_count) {
            Err(InvalidMeshError::TangentsMiscount)
        } else if self.colors.is_some_and(|x| x.len() != vertex_count) {
            Err(InvalidMeshError::ColorsMiscount)
        } else if match self.indices {
            Some(indices) => {
                let vertex_count = vertex_count
                    .try_into()
                    .map_err(InvalidMeshError::VertexUnindexible)?;
                indices.iter().any(|&x| x >= vertex_count)
            }
            None => false,
        } {
            Err(InvalidMeshError::IndexOutOfBounds)
        } else {
            Ok((vertex_count, triangle_count))
        }
    }

    /// Complete and upload the [`Mesh`].
    pub fn build(&self, _thread: &RaylibThread) -> Result<Mesh, GenMeshError> {
        let (vertex_count, triangle_count) = self.check_mesh()?;
        let raw_mesh = ffi::Mesh {
            vertexCount: vertex_count.try_into().unwrap(),
            triangleCount: triangle_count.try_into().unwrap(),
            vertices: slice_to_rl_ptr(Some(self.vertices))?,
            texcoords: slice_to_rl_ptr(Some(self.texcoords))?,
            texcoords2: slice_to_rl_ptr(self.texcoords2)?,
            normals: slice_to_rl_ptr(self.normals)?,
            tangents: slice_to_rl_ptr(self.tangents)?,
            colors: slice_to_rl_ptr(self.colors)?,
            indices: slice_to_rl_ptr(self.indices)?,
            ..Default::default()
        };
        // SAFETY: Borrowing `RaylibThread` guarantees this is the thread the resourece was created from,
        // and raw_mesh has no duplicates because it was just created.
        let mut mesh = unsafe { Mesh::from_raw(raw_mesh) };
        // SAFETY: mesh.vertices and mesh.texcoords are valid, initialized, unique, and safe to dereference.
        unsafe {
            mesh.upload(false);
        }
        Ok(mesh)
    }
}

#[cfg(test)]
mod mesh_soundness {
    use super::*;

    #[test]
    fn null_field_accessors_are_empty_not_ub() {
        // SAFETY: a zeroed ffi::Mesh has null data pointers + zero counts.
        // Accessors must return empty slices, not call slice::from_raw_parts(null, _).
        // We use WeakMesh (no-drop) so no UnloadMesh is called on a null-pointer mesh.
        let ffi_mesh: ffi::Mesh = unsafe { std::mem::zeroed() };
        let m = WeakMesh(ffi_mesh);
        assert!(
            m.vertices().is_empty(),
            "vertices() on null ptr must be empty"
        );
        assert!(
            m.normals().is_empty(),
            "normals() on null ptr must be empty"
        );
        assert!(
            m.texcoords().is_empty(),
            "texcoords() on null ptr must be empty"
        );
        assert!(
            m.tangents().is_empty(),
            "tangents() on null ptr must be empty"
        );
        assert!(m.colors().is_empty(), "colors() on null ptr must be empty");
        assert!(
            m.indices().is_empty(),
            "indices() on null ptr must be empty"
        );
        // WeakMesh does not call UnloadMesh on drop, so no cleanup needed.
    }
}
