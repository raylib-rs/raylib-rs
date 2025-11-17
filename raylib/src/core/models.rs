//! 3D Model, Mesh, and Animation

use crate::MintVec3;
use crate::core::databuf::DataBuf;
use crate::core::math::BoundingBox;
use crate::core::math::Matrix;
use crate::core::math::Transform;
use crate::core::math::{Vector2, Vector3, Vector4};
use crate::core::shaders::WeakShader;
use crate::core::texture::{Image, WeakTexture2D};
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
make_thick_wrapper! {
    /// Model, meshes, materials and animation data
    pub struct Model {
        /// Local transform matrix
        pub transform: Matrix,

        /// Number of meshes
        meshCount: i32,
        /// Number of materials
        materialCount: i32,
        /// Meshes array
        meshes: *mut WeakMesh,
        /// Materials array
        materials: *mut WeakMaterial,
        /// Mesh material number
        meshMaterial: *mut i32,

        // Animation data
        /// Number of bones
        boneCount: i32,
        /// Bones information (skeleton)
        bones: *mut BoneInfo,
        /// Bones base transformation (pose)
        bindPose: *mut Transform,
    }
    weak = WeakModel,
    raw = ffi::Model,
    drop = ffi::UnloadModel,
}
make_thick_wrapper! {
    /// Mesh, vertex data and vao/vbo
    pub struct Mesh {
        /// Number of vertices stored in arrays
        vertexCount: i32,
        /// Number of triangles stored (indexed or not)
        triangleCount: i32,

        // Vertex attributes data
        /// Vertex position (XYZ - 3 components per vertex) (shader-location = 0)
        vertices: *mut Vector3,
        /// Vertex texture coordinates (UV - 2 components per vertex) (shader-location = 1)
        texcoords: *mut Vector2,
        /// Vertex texture second coordinates (UV - 2 components per vertex) (shader-location = 5)
        texcoords2: *mut Vector2,
        /// Vertex normals (XYZ - 3 components per vertex) (shader-location = 2)
        normals: *mut Vector3,
        /// Vertex tangents (XYZW - 4 components per vertex) (shader-location = 4)
        tangents: *mut Vector4,
        /// Vertex colors (RGBA - 4 components per vertex) (shader-location = 3)
        colors: *mut Color,
        /// Vertex indices (in case vertex data comes indexed)
        indices: *mut u16,

        // Animation vertex data
        /// Animated vertex positions (after bones transformations)
        animVertices: *mut Vector3,
        /// Animated normals (after bones transformations)
        animNormals: *mut Vector3,
        /// Vertex bone ids, max 255 bone ids, up to 4 bones influence by vertex (skinning) (shader-location = 6)
        boneIds: *mut u8,
        /// Vertex bone weight, up to 4 bones influence by vertex (skinning) (shader-location = 7)
        boneWeights: *mut f32,
        /// Bones animated transformation matrices
        boneMatrices: *mut Matrix,
        /// Number of bones
        boneCount: i32,

        // OpenGL identifiers
        /// OpenGL Vertex Array Object id
        vaoId: u32,
        /// OpenGL Vertex Buffer Objects id (default vertex data)
        vboId: *mut u32,
    }
    weak = WeakMesh,
    raw = ffi::Mesh,
    drop = ffi::UnloadMesh,
}
make_thick_wrapper! {
    /// Material, includes shader and maps
    pub struct Material {
        /// Material shader
        pub shader: WeakShader,
        /// Material maps array (MAX_MATERIAL_MAPS)
        maps: *mut MaterialMap,
        /// Material generic parameters (if required)
        pub params: [f32; 4],
    }
    weak = WeakMaterial,
    raw = ffi::Material,
    drop = ffi::UnloadMaterial,
}
make_thick_wrapper! {
    /// ModelAnimation
    pub struct ModelAnimation {
        /// Number of bones
        boneCount: i32,
        /// Number of animation frames
        frameCount: i32,
        /// Bones information (skeleton)
        bones: *mut BoneInfo,
        /// Poses array by frame
        framePoses: *mut *mut Transform,
        /// Animation name
        pub name: [::std::os::raw::c_char; 32],
    }
    weak = WeakModelAnimation,
    raw = ffi::ModelAnimation,
    drop = ffi::UnloadModelAnimation,
}
make_thin_wrapper!(
    /// Bone, skeletal animation bone
    BoneInfo,
    ffi::BoneInfo,
    no_drop
);
make_thick_wrapper! {
    /// MaterialMap
    pub struct MaterialMap {
        /// Material map texture
        pub texture: WeakTexture2D,
        /// Material map color
        pub color: Color,
        /// Material map value
        pub value: f32,
    }
    raw = ffi::MaterialMap
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
        Ok(unsafe { Model::from_raw_unchecked(m) })
    }

    #[must_use]
    /// Loads model from a generated mesh
    pub fn load_model_from_mesh(
        &mut self,
        _: &RaylibThread,
        mesh: WeakMesh,
    ) -> Result<Model, LoadModelError> {
        let m = unsafe { ffi::LoadModelFromMesh(mesh.clone_raw()) };

        if m.meshes.is_null() || m.materials.is_null() {
            return Err(LoadModelError::LoadFromMeshFailed);
        }

        Ok(unsafe { Model::from_raw_unchecked(m) })
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
                m_vec.push(ModelAnimation::from_raw_unchecked(
                    *m_ptr.offset(i as isize),
                ));
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
        model: &mut Model,
        anim: &ModelAnimation,
        frame: i32,
    ) {
        unsafe {
            ffi::UpdateModelAnimation(model.clone_raw(), anim.clone_raw(), frame);
        }
    }

    /// Update model animation mesh bone matrices (GPU skinning)
    #[inline]
    pub fn update_model_animation_bones(
        &mut self,
        _: &RaylibThread,
        model: &mut Model,
        anim: &ModelAnimation,
        frame: i32,
    ) {
        unsafe {
            ffi::UpdateModelAnimationBones(model.clone_raw(), anim.clone_raw(), frame);
        }
    }
}

impl Model {
    #[inline]
    #[must_use]
    /// Local transform matrix
    fn transform(&self) -> &Matrix {
        &self.transform
    }

    #[inline]
    fn set_transform(&mut self, mat: &Matrix) {
        self.transform.clone_from(mat);
    }

    /// Meshes array
    #[inline]
    #[must_use]
    fn meshes(&self) -> &[WeakMesh] {
        unsafe { std::slice::from_raw_parts(self.meshes, self.meshCount as usize) }
    }
    // Meshes array
    #[inline]
    #[must_use]
    fn meshes_mut(&mut self) -> &mut [WeakMesh] {
        unsafe { std::slice::from_raw_parts_mut(self.meshes, self.meshCount as usize) }
    }
    /// Materials array
    #[inline]
    #[must_use]
    fn materials(&self) -> &[WeakMaterial] {
        unsafe { std::slice::from_raw_parts(self.materials, self.materialCount as usize) }
    }
    /// Materials array
    #[inline]
    #[must_use]
    fn materials_mut(&mut self) -> &mut [WeakMaterial] {
        unsafe { std::slice::from_raw_parts_mut(self.materials, self.materialCount as usize) }
    }
    #[inline]
    #[must_use]
    /// Bones information (skeleton)
    fn bones(&self) -> Option<&[BoneInfo]> {
        if self.bones.is_null() {
            return None;
        }

        Some(unsafe { std::slice::from_raw_parts(self.bones, self.boneCount as usize) })
    }
    #[inline]
    #[must_use]
    /// Bones information (skeleton)
    fn bones_mut(&mut self) -> Option<&mut [BoneInfo]> {
        if self.bones.is_null() {
            return None;
        }

        Some(unsafe { std::slice::from_raw_parts_mut(self.bones, self.boneCount as usize) })
    }
    #[inline]
    #[must_use]
    /// Bones base transformation (pose)
    fn bind_pose(&self) -> Option<&Transform> {
        if self.bindPose.is_null() {
            return None;
        }
        Some(unsafe { &*self.bindPose })
    }
    #[inline]
    #[must_use]
    /// Bones base transformation (pose)
    fn bind_pose_mut(&mut self) -> Option<&mut Transform> {
        if self.bindPose.is_null() {
            return None;
        }
        Some(unsafe { &mut *self.bindPose })
    }
    #[inline]
    #[must_use]
    /// Check model animation skeleton match
    fn is_model_animation_valid(&self, anim: &ModelAnimation) -> bool {
        unsafe { ffi::IsModelAnimationValid(self.clone_raw(), anim.clone_raw()) }
    }

    /// Check if a model is ready
    #[inline]
    #[must_use]
    fn is_model_valid(&self) -> bool {
        unsafe { ffi::IsModelValid(self.clone_raw()) }
    }

    /// Compute model bounding box limits (considers all meshes)
    #[inline]
    #[must_use]
    fn get_model_bounding_box(&self) -> BoundingBox {
        unsafe { BoundingBox::from(ffi::GetModelBoundingBox(self.clone_raw())) }
    }
    #[inline]
    /// Set material for a mesh
    fn set_model_mesh_material(
        &mut self,
        mesh_id: i32,
        material_id: i32,
    ) -> Result<(), SetMaterialError> {
        // should this be an assertion?
        if mesh_id >= self.meshCount {
            Err(SetMaterialError::MeshIdOutOfBounds)
        } else if material_id >= self.materialCount {
            Err(SetMaterialError::MaterialIdOutOfBounds)
        } else {
            unsafe { ffi::SetModelMeshMaterial(self.as_raw_mut(), mesh_id, material_id) };
            Ok(())
        }
    }
}

impl Mesh {
    /// Upload mesh vertex data in GPU and provide VAO/VBO ids
    #[inline]
    pub unsafe fn upload(&mut self, dynamic: bool) {
        unsafe { ffi::UploadMesh(self.as_raw_mut(), dynamic) };
    }
    /// Update mesh vertex data in GPU for a specific buffer index
    #[inline]
    pub unsafe fn update_buffer(&mut self, index: i32, data: &[u8], offset: i32) {
        unsafe {
            ffi::UpdateMeshBuffer(
                self.clone_raw(),
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
    pub fn vertices(&self) -> &[Vector3] {
        unsafe { std::slice::from_raw_parts(self.vertices, self.vertexCount as usize) }
    }
    /// Vertex position (XYZ - 3 components per vertex) (shader-location = 0)
    #[inline]
    #[must_use]
    pub fn vertices_mut(&mut self) -> &mut [Vector3] {
        unsafe { std::slice::from_raw_parts_mut(self.vertices, self.vertexCount as usize) }
    }
    /// Vertex normals (XYZ - 3 components per vertex) (shader-location = 2)
    #[inline]
    #[must_use]
    pub fn normals(&self) -> &[Vector3] {
        unsafe { std::slice::from_raw_parts(self.normals, self.vertexCount as usize) }
    }
    /// Vertex normals (XYZ - 3 components per vertex) (shader-location = 2)
    #[inline]
    #[must_use]
    pub fn normals_mut(&mut self) -> &mut [Vector3] {
        unsafe { std::slice::from_raw_parts_mut(self.normals, self.vertexCount as usize) }
    }
    /// Vertex tangents (XYZW - 4 components per vertex) (shader-location = 4)
    #[inline]
    #[must_use]
    pub fn tangents(&self) -> &[Vector4] {
        unsafe { std::slice::from_raw_parts(self.tangents, self.vertexCount as usize) }
    }
    /// Vertex tangents (XYZW - 4 components per vertex) (shader-location = 4)
    #[inline]
    #[must_use]
    pub fn tangents_mut(&mut self) -> &mut [Vector4] {
        unsafe { std::slice::from_raw_parts_mut(self.tangents, self.vertexCount as usize) }
    }
    /// Vertex colors (RGBA - 4 components per vertex) (shader-location = 3)
    #[inline]
    #[must_use]
    pub fn colors(&self) -> &[Color] {
        unsafe { std::slice::from_raw_parts(self.colors, self.vertexCount as usize) }
    }
    /// Vertex colors (RGBA - 4 components per vertex) (shader-location = 3)
    #[inline]
    #[must_use]
    pub fn colors_mut(&mut self) -> &mut [Color] {
        unsafe { std::slice::from_raw_parts_mut(self.colors, self.vertexCount as usize) }
    }
    /// Vertex indices (in case vertex data comes indexed)
    #[inline]
    #[must_use]
    pub fn indices(&self) -> &[u16] {
        unsafe { std::slice::from_raw_parts(self.indices as *const u16, self.vertexCount as usize) }
    }
    /// Vertex indices (in case vertex data comes indexed)
    #[inline]
    #[must_use]
    pub fn indices_mut(&mut self) -> &mut [u16] {
        unsafe { std::slice::from_raw_parts_mut(self.indices, self.vertexCount as usize) }
    }

    /// Generate polygonal mesh
    #[inline]
    #[must_use]
    pub fn gen_mesh_poly(_: &RaylibThread, sides: i32, radius: f32) -> Mesh {
        unsafe { Mesh::from_raw_unchecked(ffi::GenMeshPoly(sides, radius)) }
    }

    /// Generates plane mesh (with subdivisions).
    #[inline]
    #[must_use]
    pub fn gen_mesh_plane(
        _: &RaylibThread,
        width: f32,
        length: f32,
        res_x: i32,
        res_z: i32,
    ) -> Mesh {
        unsafe { Mesh::from_raw_unchecked(ffi::GenMeshPlane(width, length, res_x, res_z)) }
    }

    /// Generates cuboid mesh.
    #[inline]
    #[must_use]
    pub fn gen_mesh_cube(_: &RaylibThread, width: f32, height: f32, length: f32) -> Mesh {
        unsafe { Mesh::from_raw_unchecked(ffi::GenMeshCube(width, height, length)) }
    }

    /// Generates sphere mesh (standard sphere).
    #[inline]
    #[must_use]
    pub fn gen_mesh_sphere(_: &RaylibThread, radius: f32, rings: i32, slices: i32) -> Mesh {
        unsafe { Mesh::from_raw_unchecked(ffi::GenMeshSphere(radius, rings, slices)) }
    }

    /// Generates half-sphere mesh (no bottom cap).
    #[inline]
    #[must_use]
    pub fn gen_mesh_hemisphere(_: &RaylibThread, radius: f32, rings: i32, slices: i32) -> Mesh {
        unsafe { Mesh::from_raw_unchecked(ffi::GenMeshHemiSphere(radius, rings, slices)) }
    }

    /// Generates cylinder mesh.
    #[inline]
    #[must_use]
    pub fn gen_mesh_cylinder(_: &RaylibThread, radius: f32, height: f32, slices: i32) -> Mesh {
        unsafe { Mesh::from_raw_unchecked(ffi::GenMeshCylinder(radius, height, slices)) }
    }

    /// Generates torus mesh.
    #[inline]
    #[must_use]
    pub fn gen_mesh_torus(
        _: &RaylibThread,
        radius: f32,
        size: f32,
        rad_seg: i32,
        sides: i32,
    ) -> Mesh {
        unsafe { Mesh::from_raw_unchecked(ffi::GenMeshTorus(radius, size, rad_seg, sides)) }
    }

    /// Generates trefoil knot mesh.
    #[inline]
    #[must_use]
    pub fn gen_mesh_knot(
        _: &RaylibThread,
        radius: f32,
        size: f32,
        rad_seg: i32,
        sides: i32,
    ) -> Mesh {
        unsafe { Mesh::from_raw_unchecked(ffi::GenMeshKnot(radius, size, rad_seg, sides)) }
    }

    /// Generates heightmap mesh from image data.
    #[inline]
    #[must_use]
    pub fn gen_mesh_heightmap(
        _: &RaylibThread,
        heightmap: &Image,
        size: impl Into<MintVec3>,
    ) -> Mesh {
        unsafe {
            Mesh::from_raw_unchecked(ffi::GenMeshHeightmap(heightmap.clone_raw(), size.into()))
        }
    }

    /// Generates cubes-based map mesh from image data.
    #[inline]
    #[must_use]
    pub fn gen_mesh_cubicmap(
        _: &RaylibThread,
        cubicmap: &Image,
        cube_size: impl Into<MintVec3>,
    ) -> Mesh {
        unsafe {
            Mesh::from_raw_unchecked(ffi::GenMeshCubicmap(cubicmap.clone_raw(), cube_size.into()))
        }
    }

    /// Generate cone/pyramid mesh
    #[inline]
    #[must_use]
    pub fn gen_mesh_cone(_: &RaylibThread, radius: f32, height: f32, slices: i32) -> Mesh {
        unsafe { Mesh::from_raw_unchecked(ffi::GenMeshCone(radius, height, slices)) }
    }

    /// Computes mesh bounding box limits.
    #[inline]
    #[must_use]
    pub fn get_mesh_bounding_box(&self) -> BoundingBox {
        unsafe { ffi::GetMeshBoundingBox(self.clone_raw()).into() }
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
        let c_filename = CString::new(filename).unwrap();
        unsafe {
            ffi::ExportMesh(self.clone_raw(), c_filename.as_ptr());
        }
    }

    /// Export mesh as code file (.h) defining multiple arrays of vertex attributes
    #[inline]
    pub fn export_as_code(&self, filename: &str) {
        let c_filename = CString::new(filename).unwrap();
        unsafe {
            ffi::ExportMeshAsCode(self.clone_raw(), c_filename.as_ptr());
        }
    }
}

impl Material {
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
                m_vec.push(Material::from_raw_unchecked(*m_ptr.offset(i as isize)));
            }
        }
        unsafe {
            ffi::MemFree(m_ptr as *mut ::std::os::raw::c_void);
        }
        Ok(m_vec)
    }

    /// Material shader
    #[must_use]
    #[inline]
    fn shader(&self) -> &crate::shaders::WeakShader {
        unsafe { std::mem::transmute(&self.shader) }
    }
    #[must_use]
    #[inline]
    /// Material shader
    fn shader_mut(&mut self) -> &mut crate::shaders::WeakShader {
        unsafe { std::mem::transmute(&mut self.shader) }
    }
    #[must_use]
    #[inline]
    /// Material maps array (MAX_MATERIAL_MAPS)
    fn maps(&self) -> &[MaterialMap] {
        unsafe { std::slice::from_raw_parts(self.maps, consts::MAX_MATERIAL_MAPS as usize) }
    }
    #[must_use]
    #[inline]
    /// Material maps array (MAX_MATERIAL_MAPS)
    fn maps_mut(&mut self) -> &mut [MaterialMap] {
        unsafe { std::slice::from_raw_parts_mut(self.maps, consts::MAX_MATERIAL_MAPS as usize) }
    }

    /// Set texture for a material map type (MATERIAL_MAP_DIFFUSE, MATERIAL_MAP_SPECULAR...)
    #[inline]
    fn set_material_texture(
        &mut self,
        map_type: crate::consts::MaterialMapIndex,
        texture: impl AsRef<ffi::Texture2D>,
    ) {
        unsafe {
            ffi::SetMaterialTexture(
                self.as_raw_mut(),
                (map_type as u32) as i32,
                *texture.as_ref(),
            )
        }
    }

    /// Check if a material is valid (shader assigned, map textures loaded in GPU)
    #[inline]
    #[must_use]
    fn is_material_valid(&mut self) -> bool {
        unsafe { ffi::IsMaterialValid(self.clone_raw()) }
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

impl ModelAnimation {
    /// Bones information (skeleton)
    #[inline]
    #[must_use]
    fn bones(&self) -> &[BoneInfo] {
        unsafe {
            std::slice::from_raw_parts(self.bones as *const BoneInfo, self.boneCount as usize)
        }
    }

    /// Bones information (skeleton)
    #[inline]
    #[must_use]
    fn bones_mut(&mut self) -> &mut [BoneInfo] {
        unsafe {
            std::slice::from_raw_parts_mut(self.bones as *mut BoneInfo, self.boneCount as usize)
        }
    }

    #[must_use]
    /// Poses array by frame
    fn frame_poses(&self) -> Vec<&[Transform]> {
        let anim = self;
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
        let anim = self;
        unsafe {
            FramePoseIter::new(
                anim.framePoses.cast(),
                anim.frameCount as usize,
                anim.boneCount as usize,
            )
        }
    }

    #[must_use]
    /// Poses array by frame
    fn frame_poses_mut(&mut self) -> Vec<&mut [Transform]> {
        let anim = self;
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
        let anim = self;
        unsafe {
            FramePoseIterMut::new(
                anim.framePoses.cast(),
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
        unsafe { std::mem::transmute(&self.texture) }
    }
    /// Material map texture
    #[inline]
    #[must_use]
    pub fn texture_mut(&mut self) -> &mut crate::texture::WeakTexture2D {
        unsafe { std::mem::transmute(&mut self.texture) }
    }

    /// Material map color
    #[inline]
    #[must_use]
    pub fn color(&self) -> &Color {
        unsafe { std::mem::transmute(&self.color) }
    }
    /// Material map color
    #[inline]
    #[must_use]
    pub fn color_mut(&mut self) -> &mut Color {
        unsafe { std::mem::transmute(&mut self.color) }
    }

    /// Material map value
    #[inline]
    #[must_use]
    pub fn value(&self) -> &f32 {
        unsafe { std::mem::transmute(&self.value) }
    }
    /// Material map value
    #[inline]
    #[must_use]
    pub fn value_mut(&mut self) -> &mut f32 {
        unsafe { std::mem::transmute(&mut self.value) }
    }
}

impl RaylibHandle {
    /// Load default material (Supports: DIFFUSE, SPECULAR, NORMAL maps)
    #[inline]
    #[must_use]
    pub fn load_material_default(&self, _: &RaylibThread) -> WeakMaterial {
        unsafe { Material::from_raw_unchecked(ffi::LoadMaterialDefault()).make_weak() }
    }

    /// Weak materials will leak memory if they are not unlaoded
    /// Unload material from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_material(&mut self, _: &RaylibThread, material: WeakMaterial) {
        unsafe { _ = Material::from_weak(material) }
    }

    /// Weak models will leak memory if they are not unlaoded
    /// Unload model from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_model(&mut self, _: &RaylibThread, model: WeakModel) {
        unsafe { _ = Model::from_weak(model) }
    }

    /// Weak model_animations will leak memory if they are not unlaoded
    /// Unload model_animation from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_model_animation(
        &mut self,
        _: &RaylibThread,
        model_animation: WeakModelAnimation,
    ) {
        unsafe { _ = ModelAnimation::from_weak(model_animation) }
    }

    /// Weak meshs will leak memory if they are not unlaoded
    /// Unload mesh from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_mesh(&mut self, _: &RaylibThread, mesh: WeakMesh) {
        unsafe { _ = Mesh::from_weak(mesh) }
    }
}

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
        let mut mesh = unsafe { Mesh::from_raw_unchecked(raw_mesh) };
        // SAFETY: mesh.vertices and mesh.texcoords are valid, initialized, unique, and safe to dereference.
        unsafe {
            mesh.upload(false);
        }
        Ok(mesh)
    }
}
