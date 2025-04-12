//! 3D Model, Mesh, and Animation

use crate::core::math::{BoundingBox, Vector3};
use crate::core::texture::Image;
use crate::core::{RaylibHandle, RaylibThread};
use crate::{consts, ffi, error::{LoadMaterialError, LoadModelAnimError, LoadModelError, SetMaterialError}};
use std::ffi::CString;
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
    /// ModelAnimation
    ModelAnimation,
    ffi::ModelAnimation,
    |anim: ffi::ModelAnimation| {
        for i in 0..anim.keyframeCount {
            ffi::MemFree(*anim.keyframePoses.offset(i as isize) as *mut c_void);
        }
        ffi::MemFree(anim.keyframePoses as *mut c_void);
    }
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

impl RaylibHandle {
    /// Loads model from files (mesh and material).
    // #[inline]
    pub fn load_model(&mut self, _: &RaylibThread, filename: &str) -> Result<Model, LoadModelError> {
        let c_filename = CString::new(filename).unwrap();
        let m = unsafe { ffi::LoadModel(c_filename.as_ptr()) };
        if m.meshes.is_null()
            && m.materials.is_null()
            && m.skeleton.bones.is_null()
            && m.skeleton.bindPose.is_null()
        {
            return Err(LoadModelError::LoadFromFileFailed { path: filename.into() });
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
    ) -> Result<Vec<ModelAnimation>, LoadModelAnimError> {
        let c_filename = CString::new(filename).unwrap();
        let mut m_size = 0;
        let m_ptr = unsafe { ffi::LoadModelAnimations(c_filename.as_ptr(), &mut m_size) };
        if m_size <= 0 {
            return Err(LoadModelAnimError::NoAnimationsLoaded { path: filename.into() });
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
        frame: f32,
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
    /// Convert into a [`WeakModel`] that does not own the underlying resource.
    ///
    /// # Safety
    ///
    /// The caller becomes responsible for ensuring the returned `WeakModel`
    /// does not outlive the GPU resources it references, and for explicitly
    /// unloading the model via [`RaylibHandle::unload_model`] when finished.
    /// Failing to do so will leak GPU memory.
    pub unsafe fn make_weak(self) -> WeakModel {
        let m = WeakModel(self.0);
        std::mem::forget(self);
        m
    }
}

pub trait RaylibModel: AsRef<ffi::Model> + AsMut<ffi::Model> {
    /// Local transform matrix
    fn transform(&self) -> &crate::math::Matrix {
        unsafe { std::mem::transmute(&self.as_ref().transform) }
    }

    fn set_transform(&mut self, mat: &crate::math::Matrix) {
        self.as_mut().transform = mat.into();
    }

    /// Meshes array
    #[inline]
    fn meshes(&self) -> &[WeakMesh] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().meshes as *const WeakMesh,
                self.as_ref().meshCount as usize,
            )
        }
    }
    // Meshes array
    #[inline]
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
    fn materials_mut(&mut self) -> &mut [WeakMaterial] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().materials as *mut WeakMaterial,
                self.as_mut().materialCount as usize,
            )
        }
    }

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
    /// Bones base transformation (pose)
    fn bind_pose(&self) -> Option<&crate::math::Transform> {
        if self.as_ref().bindPose.is_null() {
            return None;
        }
        Some(unsafe {
            std::slice::from_raw_parts(
                self.as_ref().skeleton.bindPose as *const crate::math::Transform,
                self.as_ref().skeleton.boneCount as usize,
            )
        })
    }

    /// Bones base transformation (pose)
    fn bind_pose_mut(&mut self) -> Option<&mut crate::math::Transform> {
        if self.as_ref().bindPose.is_null() {
            return None;
        }
        Some(unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().skeleton.bindPose as *mut crate::math::Transform,
                self.as_ref().skeleton.boneCount as usize,
            )
        })
    }

    /// Check model animation skeleton match
    #[inline]
    fn is_model_animation_valid(&self, anim: &ModelAnimation) -> bool {
        unsafe { ffi::IsModelAnimationValid(*self.as_ref(), anim.0) }
    }

    /// Check if a model is ready
    #[inline]
    fn is_model_valid(&self) -> bool {
        unsafe { ffi::IsModelValid(*self.as_ref()) }
    }

    /// Compute model bounding box limits (considers all meshes)
    #[inline]
    fn get_model_bounding_box(&self) -> BoundingBox {
        unsafe { BoundingBox::from(ffi::GetModelBoundingBox(*self.as_ref())) }
    }

    /// Set material for a mesh
    fn set_model_mesh_material(&mut self, mesh_id: i32, material_id: i32) -> Result<(), SetMaterialError> {
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
    /// Convert into a [`WeakMesh`] that does not own the underlying resource.
    ///
    /// # Safety
    ///
    /// The caller becomes responsible for ensuring the returned `WeakMesh`
    /// does not outlive the GPU resources it references, and for explicitly
    /// unloading the mesh via [`RaylibHandle::unload_mesh`] when finished.
    /// Failing to do so will leak GPU memory.
    pub unsafe fn make_weak(self) -> WeakMesh {
        let m = WeakMesh(self.0);
        std::mem::forget(self);
        m
    }
}
pub trait RaylibMesh: AsRef<ffi::Mesh> + AsMut<ffi::Mesh> {
    /// Upload mesh vertex data in GPU and provide VAO/VBO ids
    #[inline]
    unsafe fn upload(&mut self, dynamic: bool) {
        ffi::UploadMesh(self.as_mut(), dynamic);
    }
    /// Update mesh vertex data in GPU for a specific buffer index
    #[inline]
    unsafe fn update_buffer<A>(&mut self, index: i32, data: &[u8], offset: i32) {
        ffi::UpdateMeshBuffer(
            *self.as_ref(),
            index,
            data.as_ptr() as *const c_void,
            data.len() as i32,
            offset,
        );
    }
    /// Vertex position (XYZ - 3 components per vertex) (shader-location = 0)
    #[inline]
    fn vertices(&self) -> &[Vector3] {
        NonNull::new(self.as_ref().vertices.cast()).map_or(&[], |data| unsafe {
            NonNull::slice_from_raw_parts(data, self.as_ref().vertexCount as usize).as_ref()
        })
    }
    /// Vertex position (XYZ - 3 components per vertex) (shader-location = 0)
    #[inline]
    fn vertices_mut(&mut self) -> &mut [Vector3] {
        NonNull::new(self.as_ref().vertices.cast()).map_or(&mut [], |data| unsafe {
            NonNull::slice_from_raw_parts(data, self.as_ref().vertexCount as usize).as_mut()
        })
    }
    /// Vertex normals (XYZ - 3 components per vertex) (shader-location = 2)
    #[inline]
    fn normals(&self) -> &[Vector3] {
        NonNull::new(self.as_ref().normals.cast()).map_or(&[], |data| unsafe {
            NonNull::slice_from_raw_parts(data, self.as_ref().vertexCount as usize).as_ref()
        })
    }
    /// Vertex normals (XYZ - 3 components per vertex) (shader-location = 2)
    #[inline]
    fn normals_mut(&mut self) -> &mut [Vector3] {
        NonNull::new(self.as_ref().normals.cast()).map_or(&mut [], |data| unsafe {
            NonNull::slice_from_raw_parts(data, self.as_ref().vertexCount as usize).as_mut()
        })
    }
    /// Vertex tangents (XYZW - 4 components per vertex) (shader-location = 4)
    #[inline]
    fn tangents(&self) -> &[Vector3] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().tangents as *const Vector3,
                self.as_ref().vertexCount as usize,
            )
        }
    }
    /// Vertex tangents (XYZW - 4 components per vertex) (shader-location = 4)
    #[inline]
    fn tangents_mut(&mut self) -> &mut [Vector3] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().tangents as *mut Vector3,
                self.as_mut().vertexCount as usize,
            )
        }
    }
    /// Vertex colors (RGBA - 4 components per vertex) (shader-location = 3)
    #[inline]
    fn colors(&self) -> &[crate::color::Color] {
        NonNull::new(self.as_ref().colors as *mut crate::color::Color).map_or(&[], |data| unsafe {
            NonNull::slice_from_raw_parts(data, self.as_ref().vertexCount as usize).as_ref()
        })
    }
    /// Vertex colors (RGBA - 4 components per vertex) (shader-location = 3)
    #[inline]
    fn colors_mut(&mut self) -> &mut [crate::color::Color] {
        NonNull::new(self.as_ref().colors as *mut crate::color::Color).map_or(
            &mut [],
            |data| unsafe {
                NonNull::slice_from_raw_parts(data, self.as_ref().vertexCount as usize).as_mut()
            },
        )
    }
    /// Vertex indices (in case vertex data comes indexed)
    #[inline]
    fn indicies(&self) -> &[u16] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().indices as *const u16,
                self.as_ref().vertexCount as usize,
            )
        }
    }
    /// Vertex indices (in case vertex data comes indexed)
    #[inline]
    fn indicies_mut(&mut self) -> &mut [u16] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().indices as *mut u16,
                self.as_mut().vertexCount as usize,
            )
        }
    }

    /// Generate polygonal mesh
    #[inline]
    fn gen_mesh_poly(_: &RaylibThread, sides: i32, radius: f32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshPoly(sides, radius)) }
    }

    /// Generates plane mesh (with subdivisions).
    #[inline]
    fn gen_mesh_plane(_: &RaylibThread, width: f32, length: f32, res_x: i32, res_z: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshPlane(width, length, res_x, res_z)) }
    }

    /// Generates cuboid mesh.
    #[inline]
    fn gen_mesh_cube(_: &RaylibThread, width: f32, height: f32, length: f32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshCube(width, height, length)) }
    }

    /// Generates sphere mesh (standard sphere).
    #[inline]
    fn gen_mesh_sphere(_: &RaylibThread, radius: f32, rings: i32, slices: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshSphere(radius, rings, slices)) }
    }

    /// Generates half-sphere mesh (no bottom cap).
    #[inline]
    fn gen_mesh_hemisphere(_: &RaylibThread, radius: f32, rings: i32, slices: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshHemiSphere(radius, rings, slices)) }
    }

    /// Generates cylinder mesh.
    #[inline]
    fn gen_mesh_cylinder(_: &RaylibThread, radius: f32, height: f32, slices: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshCylinder(radius, height, slices)) }
    }

    /// Generates torus mesh.
    #[inline]
    fn gen_mesh_torus(_: &RaylibThread, radius: f32, size: f32, rad_seg: i32, sides: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshTorus(radius, size, rad_seg, sides)) }
    }

    /// Generates trefoil knot mesh.
    #[inline]
    fn gen_mesh_knot(_: &RaylibThread, radius: f32, size: f32, rad_seg: i32, sides: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshKnot(radius, size, rad_seg, sides)) }
    }

    /// Generates heightmap mesh from image data.
    #[inline]
    fn gen_mesh_heightmap(
        _: &RaylibThread,
        heightmap: &Image,
        size: impl Into<ffi::Vector3>,
    ) -> Mesh {
        unsafe { Mesh(ffi::GenMeshHeightmap(heightmap.0, size.into())) }
    }

    /// Generates cubes-based map mesh from image data.
    #[inline]
    fn gen_mesh_cubicmap(
        _: &RaylibThread,
        cubicmap: &Image,
        cube_size: impl Into<ffi::Vector3>,
    ) -> Mesh {
        unsafe { Mesh(ffi::GenMeshCubicmap(cubicmap.0, cube_size.into())) }
    }

    /// Generate cone/pyramid mesh
    #[inline]
    fn gen_mesh_cone(_: &RaylibThread, radius: f32, height: f32, slices: i32) -> Mesh {
        unsafe { Mesh(ffi::GenMeshCone(radius, height, slices)) }
    }

    /// Computes mesh bounding box limits.
    #[inline]
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
    /// Convert into a [`WeakMaterial`] that does not own the underlying resource.
    ///
    /// # Safety
    ///
    /// The caller becomes responsible for ensuring the returned `WeakMaterial`
    /// does not outlive the GPU resources it references, and for explicitly
    /// unloading the material via [`RaylibHandle::unload_material`] when
    /// finished. Failing to do so will leak GPU memory.
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
            return Err(LoadMaterialError::NoneLoaded { path: filename.into() });
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
    fn shader(&self) -> &crate::shaders::WeakShader {
        unsafe { std::mem::transmute(&self.as_ref().shader) }
    }

    /// Material shader
    fn shader_mut(&mut self) -> &mut crate::shaders::WeakShader {
        unsafe { std::mem::transmute(&mut self.as_mut().shader) }
    }

    /// Material maps array (MAX_MATERIAL_MAPS)
    fn maps(&self) -> &[MaterialMap] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().maps as *const MaterialMap,
                consts::MAX_MATERIAL_MAPS as usize,
            )
        }
    }

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
    fn is_material_valid(&mut self) -> bool {
        unsafe { ffi::IsMaterialValid(*self.as_ref()) }
    }
}

#[derive(Debug, Clone)]
pub struct FramePoseIter<'a> {
    iter: std::slice::Iter<'a, Option<&'a [crate::math::Transform]>>,
    bone_count: usize,
}
impl<'a> FramePoseIter<'a> {
    unsafe fn new(frame_poses: *mut *mut ffi::Transform, frame_count: usize, bone_count: usize) -> Self {
        // No new items are being created that get dropped here, these are just changes in perspective of how to borrow-check the pointers.
        assert!(!frame_poses.is_null(), "frame pose array cannot be null");
        assert!(frame_poses.is_aligned(), "frame pose array must be aligned");
        let frame_poses = frame_poses.cast::<Option<&'a [crate::math::Transform]>>();
        let iter = unsafe { std::slice::from_raw_parts(frame_poses, frame_count) }.iter();
        Self { iter, bone_count }
    }
    fn func(tf: &Option<&'a [crate::math::Transform]>, bone_count: usize) -> &'a [crate::math::Transform] {
        unsafe { std::slice::from_raw_parts(tf.expect("frame pose transform cannot be null").as_ptr(), bone_count) }
    }
}
impl<'a> Iterator for FramePoseIter<'a> {
    type Item = &'a [crate::math::Transform];

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
        self.iter.next_back().map(move |tf| Self::func(tf, bone_count))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter.nth_back(n).map(move |tf| Self::func(tf, bone_count))
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
    iter: std::slice::IterMut<'a, Option<&'a mut [crate::math::Transform]>>,
    bone_count: usize,
}
impl<'a> FramePoseIterMut<'a> {
    unsafe fn new(frame_poses: *mut *mut ffi::Transform, frame_count: usize, bone_count: usize) -> Self {
        // No new items are being created that get dropped here, these are just changes in perspective of how to borrow-check the pointers.
        assert!(!frame_poses.is_null(), "frame pose array cannot be null");
        assert!(frame_poses.is_aligned(), "frame pose array must be aligned");
        let frame_poses = frame_poses.cast::<Option<&'a mut [crate::math::Transform]>>();
        let iter = unsafe { std::slice::from_raw_parts_mut(frame_poses, frame_count) }.iter_mut();
        Self { iter, bone_count }
    }
    fn func(tf: &mut Option<&'a mut [crate::math::Transform]>, bone_count: usize) -> &'a mut [crate::math::Transform] {
        unsafe { std::slice::from_raw_parts_mut(tf.as_mut().expect("frame pose transform cannot be null").as_mut_ptr(), bone_count) }
    }
}
impl<'a> Iterator for FramePoseIterMut<'a> {
    type Item = &'a mut [crate::math::Transform];

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
        self.iter.next_back().map(move |tf| Self::func(tf, bone_count))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let bone_count = self.bone_count;
        self.iter.nth_back(n).map(move |tf| Self::func(tf, bone_count))
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
    /// Convert into a [`WeakModelAnimation`] that does not own the underlying
    /// resource.
    ///
    /// # Safety
    ///
    /// The caller becomes responsible for ensuring the returned
    /// `WeakModelAnimation` does not outlive the underlying data, and for
    /// explicitly unloading the animation via
    /// [`RaylibHandle::unload_model_animation`] when finished. Failing to do
    /// so will leak memory.
    pub unsafe fn make_weak(self) -> WeakModelAnimation {
        let m = WeakModelAnimation(self.0);
        std::mem::forget(self);
        m
    }
}

pub trait RaylibModelAnimation: AsRef<ffi::ModelAnimation> + AsMut<ffi::ModelAnimation> {
    /// Bones information (skeleton)
    #[inline]
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
    fn bones_mut(&mut self) -> &mut [BoneInfo] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().bones as *mut BoneInfo,
                self.as_mut().boneCount as usize,
            )
        }
    }

    /// Poses array by frame
    fn frame_poses(&self) -> Vec<&[crate::math::Transform]> {
        let anim = self.as_ref();
        let mut top = Vec::with_capacity(anim.keyframeCount as usize);

        for i in 0..anim.keyframeCount {
            top.push(unsafe {
                std::slice::from_raw_parts(
                    *(anim.keyframePoses.offset(i as isize)
                        as *const *const crate::math::Transform),
                    anim.boneCount as usize,
                )
            });
        }

        top
    }
    fn frame_poses_iter<'a>(&'a self) -> FramePoseIter<'a> {
        let anim = self.as_ref();
        unsafe { FramePoseIter::new(anim.framePoses, anim.frameCount as usize, anim.boneCount as usize) }
    }

    /// Poses array by frame
    fn frame_poses_mut(&mut self) -> Vec<&mut [crate::math::Transform]> {
        let anim = self.as_ref();
        let mut top = Vec::with_capacity(anim.keyframeCount as usize);

        for i in 0..anim.keyframeCount {
            top.push(unsafe {
                std::slice::from_raw_parts_mut(
                    *(anim.keyframePoses.offset(i as isize) as *mut *mut crate::math::Transform),
                    anim.boneCount as usize,
                )
            });
        }

        top
    }
    fn frame_poses_iter_mut<'a>(&'a mut self) -> FramePoseIterMut<'a> {
        let anim = self.as_ref();
        unsafe { FramePoseIterMut::new(anim.framePoses, anim.frameCount as usize, anim.boneCount as usize) }
    }
}

impl MaterialMap {
    /// Material map texture
    #[inline]
    pub fn texture(&self) -> &crate::texture::WeakTexture2D {
        unsafe { std::mem::transmute(&self.0.texture) }
    }
    /// Material map texture
    #[inline]
    pub fn texture_mut(&mut self) -> &mut crate::texture::WeakTexture2D {
        unsafe { std::mem::transmute(&mut self.0.texture) }
    }

    /// Material map color
    #[inline]
    pub fn color(&self) -> &crate::color::Color {
        unsafe { std::mem::transmute(&self.0.color) }
    }
    /// Material map color
    #[inline]
    pub fn color_mut(&mut self) -> &mut crate::color::Color {
        unsafe { std::mem::transmute(&mut self.0.color) }
    }

    /// Material map value
    #[inline]
    pub fn value(&self) -> &f32 {
        unsafe { std::mem::transmute(&self.0.value) }
    }
    /// Material map value
    #[inline]
    pub fn value_mut(&mut self) -> &mut f32 {
        unsafe { std::mem::transmute(&mut self.0.value) }
    }
}

impl RaylibHandle {
    /// Load default material (Supports: DIFFUSE, SPECULAR, NORMAL maps)
    #[inline]
    pub fn load_material_default(&self, _: &RaylibThread) -> WeakMaterial {
        WeakMaterial(unsafe { ffi::LoadMaterialDefault() })
    }

    /// Weak materials will leak memeory if they are not unlaoded
    /// Unload material from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_material(&mut self, _: &RaylibThread, material: WeakMaterial) {
        {
            ffi::UnloadMaterial(*material.as_ref())
        }
    }

    /// Weak models will leak memeory if they are not unlaoded
    /// Unload model from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_model(&mut self, _: &RaylibThread, model: WeakModel) {
        {
            ffi::UnloadModel(*model.as_ref())
        }
    }

    /// Weak model_animations will leak memeory if they are not unlaoded
    /// Unload model_animation from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_model_animation(
        &mut self,
        _: &RaylibThread,
        model_animation: WeakModelAnimation,
    ) {
        // Mirror the per-animation inner cleanup from raylib 6.0's
        // `UnloadModelAnimations` without freeing the struct itself.
        let anim = *model_animation.as_ref();
        for i in 0..anim.keyframeCount {
            ffi::MemFree(*anim.keyframePoses.offset(i as isize) as *mut c_void);
        }
        ffi::MemFree(anim.keyframePoses as *mut c_void);
    }

    /// Weak meshs will leak memeory if they are not unlaoded
    /// Unload mesh from GPU memory (VRAM)
    #[inline]
    pub unsafe fn unload_mesh(&mut self, _: &RaylibThread, mesh: WeakMesh) {
        {
            ffi::UnloadMesh(*mesh.as_ref())
        }
    }
}

#[cfg(test)]
mod mesh_tests {
    use super::{RaylibMesh, WeakMesh};
    use crate::ffi;
    use crate::math::Vector4;

    /// Regression test for the pre-6.0.0 tangents bug: raylib stores
    /// tangents as `float[4 * vertexCount]` (XYZW), but the safe wrapper
    /// used to cast the buffer to `*const Vector3` and read 3/4 of each
    /// tangent with wrong component alignment. Now returns `&[Vector4]`
    /// with `vertexCount` elements, each holding the full XYZW.
    #[test]
    fn tangents_returns_vector4_per_vertex() {
        const N: i32 = 3;
        let mut buf: Vec<f32> = vec![
            1.0, 2.0, 3.0, 4.0, //
            5.0, 6.0, 7.0, 8.0, //
            9.0, 10.0, 11.0, 12.0,
        ];

        // SAFETY: `ffi::Mesh` is POD-shaped (primitives + raw pointers), so
        // `zeroed` yields a valid value. `WeakMesh` has a no-op `Drop`, so
        // nothing tries to free the borrowed buffer.
        let mesh = unsafe {
            let mut m: ffi::Mesh = std::mem::zeroed();
            m.vertexCount = N;
            m.tangents = buf.as_mut_ptr();
            WeakMesh::from_raw(m)
        };

        let tangents = mesh.tangents();
        assert_eq!(tangents.len(), N as usize);
        assert_eq!(
            tangents[0],
            Vector4 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                w: 4.0
            }
        );
        assert_eq!(
            tangents[1],
            Vector4 {
                x: 5.0,
                y: 6.0,
                z: 7.0,
                w: 8.0
            }
        );
        assert_eq!(
            tangents[2],
            Vector4 {
                x: 9.0,
                y: 10.0,
                z: 11.0,
                w: 12.0
            }
        );
    }

    #[test]
    fn tangents_empty_when_pointer_null() {
        let mesh = unsafe {
            let mut m: ffi::Mesh = std::mem::zeroed();
            m.vertexCount = 42;
            m.tangents = std::ptr::null_mut();
            WeakMesh::from_raw(m)
        };
        assert!(mesh.tangents().is_empty());
    }
}
