use crate::gui::DebugGUI;
use crate::gui::DebugGUIFormat;
use crate::material::pbr::PBRParams;
use crate::material::PBRTextures;
use crate::material::PBR;
use crate::renderer::RenderScene;
use cgmath::Basis3;
use cgmath::Matrix3;
use cgmath::Matrix4;
use cgmath::MetricSpace;
use cgmath::Quaternion;
use cgmath::Rad;
use cgmath::Vector3;
use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use rayon::prelude::*;
use russimp::material::PropertyTypeInfo::FloatArray;
use russimp::scene::PostProcess;
use russimp::scene::Scene;
use std::error::Error;
use std::path::Path;

use crate::vertex::Vertex;

/// Section of a [`PbrModel`]
///
/// Models often consist of multiple smaller models, I am calling them segments.
pub struct PbrModelSegment {
    material: PBR,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
}

impl PbrModelSegment {
    pub fn new(
        vertex_buffer: VertexBuffer<Vertex>,
        index_buffer: IndexBuffer<u32>,
        material: PBR,
    ) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            material,
        }
    }

    /// Sets the translation matrix for all the vertices of the model.
    ///
    /// You shouldn't need to use this.
    pub fn build_matrix(&mut self, model: Matrix4<f32>) {
        self.material.set_model_matrix(model);
    }

    pub fn render<'a>(&'a self, scene: &mut RenderScene<'a>) {
        scene.publish(&self.vertex_buffer, &self.index_buffer, &self.material);
    }

    pub fn get_material(&self) -> &PBR {
        &self.material
    }
    pub fn get_material_mut(&mut self) -> &mut PBR {
        &mut self.material
    }
    pub fn set_material(&mut self, material: PBR) {
        self.material = material;
    }

    pub fn get_vertex_buffer(&self) -> &VertexBuffer<Vertex> {
        &self.vertex_buffer
    }

    pub fn get_index_buffer(&self) -> &IndexBuffer<u32> {
        &self.index_buffer
    }
}

impl Clone for PbrModelSegment {
    fn clone(&self) -> Self {
        let facade = self.vertex_buffer.get_context();
        let index_data = self.index_buffer.read().unwrap();
        let vertex_data = self.vertex_buffer.read().unwrap();

        let index_buffer =
            IndexBuffer::new(facade, self.index_buffer.get_primitives_type(), &index_data).unwrap();
        let vertex_buffer = VertexBuffer::new(facade, &vertex_data).unwrap();
        let material = self.material.clone();

        Self {
            index_buffer,
            vertex_buffer,
            material,
        }
    }
}

impl DebugGUI for PbrModelSegment {
    fn debug(&mut self, ui: &mut egui::Ui) {
        self.material.debug(ui);
    }
}

/// A model that will be rendered using Physically Based Rendering
///
/// When the `PbrModel` is constructed, it will consist of multiple segments. Each segment has its
/// own index and vertex buffers and has its own [`PBR`] material which defines how the whole
/// segment should be colored. The `PbrModel` also holds the position and rotation of the model (in
/// world space). This is how you will move the model around in the scene. To render the model you
/// will call `render` on the object and pass in a [`RenderScene`] which will control when the
/// model should be rendered.
///
/// # Example
///
/// ```
/// // Load the model from the file system
/// let mut model = PbrModel::load_from_fs("cube.glb", display, material);
///
/// // Rotate and move the model
/// model.relative_move([1.0, 0.0, 0.0]);
/// model.relative_rotate([0.0, 0.0, Rad(std::f32::consts::PI / 2.0)]);
///
/// // scene is a RenderScene
/// // Submit the model to be rendered
/// model.render(&mut scene);
/// ```
#[derive(Clone)]
pub struct PbrModel {
    position: Vector3<f32>,
    rotation_matrix: Matrix4<f32>,
    segments: Vec<PbrModelSegment>,
}

impl PbrModel {
    /// Loads a model file from the file system
    ///
    /// This should primarily be used for loading glTF 2.0 (.glb) files as they support PBR
    /// materials. It does load wavefront (.obj) files, but the material will not look right due to
    /// wavefront not supporting pbr materials.
    pub fn load_from_fs<P>(
        path: P,
        facade: &impl Facade,
        material: PBR,
    ) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let scene = Scene::from_file(
            path.as_ref()
                .as_os_str()
                .to_str()
                .ok_or("file path could not be made into a string")?,
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
                PostProcess::FlipWindingOrder,
                PostProcess::MakeLeftHanded,
                PostProcess::OptimizeMeshes,
                // Quick fix, should change later
                PostProcess::PreTransformVertices,
            ],
        )?;

        let mut segments = Vec::new();

        for mesh in scene.meshes {
            let vertices = (0..mesh.vertices.len())
                .into_par_iter()
                .map(|index| {
                    let vertex = mesh.vertices[index as usize];
                    let position: [f32; 3] = [vertex.x, vertex.y, vertex.z];
                    let normal_vec = mesh.normals[index as usize];
                    let normal = [normal_vec.x, normal_vec.y, normal_vec.z];
                    let tex_coords = match mesh.texture_coords[0].as_ref() {
                        Some(texture_coords) => {
                            let vec3 = texture_coords[index as usize];
                            [vec3.x, vec3.y]
                        }
                        None => [0.0; 2],
                    };

                    return Vertex {
                        position,
                        normal,
                        tex_coords,
                        ..Default::default()
                    };
                })
                .collect::<Vec<_>>();

            let indices = mesh
                .faces
                .into_par_iter()
                .flat_map(|face| face.0.into_par_iter())
                .collect::<Vec<_>>();

            let index_buffer =
                IndexBuffer::new(facade, glium::index::PrimitiveType::TrianglesList, &indices)?;
            let vertex_buffer = VertexBuffer::new(facade, &vertices)?;

            let mut material = material.clone();
            let mut basic_mat = PBRParams::default();

            let scene_material = &scene.materials[mesh.material_index as usize];
            for property in &scene_material.properties {
                // gltf
                if property.key == "$clr.base" {
                    if let FloatArray(data) = &property.data {
                        basic_mat.set_albedo([data[0], data[1], data[2]]);
                    }
                } else if property.key == "$mat.roughnessFactor" {
                    if let FloatArray(data) = &property.data {
                        basic_mat.set_roughness(data[0]);
                    }
                } else if property.key == "$mat.metallicFactor" {
                    if let FloatArray(data) = &property.data {
                        basic_mat.set_metallic(data[0]);
                    }
                }

                // obj
                if property.key == "$clr.diffuse" {
                    if let FloatArray(data) = &property.data {
                        basic_mat.set_albedo([data[0], data[1], data[2]]);
                    }
                } else if property.key == "$mat.shininess" {
                    if let FloatArray(data) = &property.data {
                        let shininess = data[0].min(900.0) / 900.0;
                        let roughness = (1.0 - shininess).max(0.05);

                        basic_mat.set_roughness(roughness);
                    }
                }
            }
            material.set_pbr_params(PBRTextures::from_params(basic_mat, facade));

            segments.push(PbrModelSegment::new(vertex_buffer, index_buffer, material));
        }

        Ok(Self {
            position: [0.0; 3].into(),
            rotation_matrix: Matrix4::from_angle_x(Rad(0.0)),
            segments,
        })
    }

    /// Makes a model out of vertex and index buffers
    ///
    /// Just pass in the vertex and index buffers along with a PBR material and it will return a
    /// PbrModel
    pub fn load_from_mem(
        vertex_buffer: VertexBuffer<Vertex>,
        index_buffer: IndexBuffer<u32>,
        material: PBR,
    ) -> Self {
        let segments = vec![PbrModelSegment::new(
            vertex_buffer,
            index_buffer,
            material.clone(),
        )];

        Self {
            position: [0.0; 3].into(),
            rotation_matrix: Matrix4::from_angle_x(Rad(0.0)),
            segments,
        }
    }

    /// Submits the PbrModel to the [`RenderScene`] to be rendered
    ///
    /// Passes the model to the [`RenderScene`]. The [`RenderScene`]
    /// holds this struct by reference, so this struct must outlive the [`RenderScene`].
    ///
    /// [`RenderScene`]: crate::renderer::RenderScene
    pub fn render<'a>(&'a self, scene: &mut RenderScene<'a>) {
        let camera: Vector3<f32> = (*scene.get_scene_data().get_camera_pos()).into();
        let object: Vector3<f32> = self.position.into();

        // Lod
        let distance = object.distance(camera);

        if distance >= 5.0 {
            // Change LOD
        }

        for item in &self.segments {
            item.render(scene);
        }
    }

    /// Rebuilds the transformation matrices for the model
    ///
    /// You likely won't need to use this. This needs to be called when the position or rotation is modified.
    /// When modifying the position or rotation with a function like `relative_move` or
    /// `relative_rotate` this will automatically be called.
    pub fn build_matrix(&mut self) {
        let translation = Matrix4::from_translation(self.position);

        let model = translation * self.rotation_matrix;

        for segment in &mut self.segments {
            segment.build_matrix(model.clone());
        }
    }

    pub fn set_rotation_matrix(&mut self, mat: [[f32; 4]; 4]) {
        let rotation_mat = Matrix4::from(mat);
        let translation = Matrix4::from_translation(self.position);

        let model = translation * rotation_mat;

        for segment in &mut self.segments {
            segment.build_matrix(model.clone());
        }
    }

    pub fn set_rotation_axis_angle(
        &mut self,
        axis: impl Into<Vector3<f32>>,
        angle: impl Into<Rad<f32>>,
    ) {
        let rotation_mat = Matrix4::from_axis_angle(axis.into(), angle);
        let translation = Matrix4::from_translation(self.position);

        let model = translation * rotation_mat;

        for segment in &mut self.segments {
            segment.build_matrix(model.clone());
        }
    }

    /// Moves the model
    ///
    /// Used to translate the object relative to its current position.
    ///
    /// # Example
    ///
    /// ```
    /// model.relative_move([1.0, 0.0, 0.0]);
    /// ```
    pub fn relative_move(&mut self, position: impl Into<Vector3<f32>>) {
        self.position = self.position + position.into();
        self.build_matrix();
    }

    /// Moves the model
    ///
    /// Used to set the translation.
    pub fn set_translation(&mut self, position: impl Into<Vector3<f32>>) {
        self.position = position.into();
        self.build_matrix();
    }

    /// Rotates the model
    ///
    /// Used to rotate the object relative to its current rotation.
    ///
    /// # Example
    ///
    /// ```
    /// model.relative_rotate([Rad(0.0), Rad(0.0), Rad(std::f32::consts::PI/2.0)]);
    /// ```
    pub fn relative_rotate(&mut self, rotation: impl Into<Vector3<Rad<f32>>>) {
        let rotation = rotation.into();
        self.rotation_matrix = Matrix4::from_angle_x(rotation.x)
            * Matrix4::from_angle_y(rotation.y)
            * Matrix4::from_angle_z(rotation.z);
        self.build_matrix();
    }

    pub fn set_rotation(&mut self, rotation: impl Into<Vector3<Rad<f32>>>) {
        let rotation = rotation.into();
        self.rotation_matrix = Matrix4::from_angle_x(rotation.x)
            * Matrix4::from_angle_y(rotation.y)
            * Matrix4::from_angle_z(rotation.z);
        self.build_matrix();
    }

    pub fn set_rotation_euler(&mut self, yaw: Rad<f32>, pitch: Rad<f32>, roll: Rad<f32>) {
        // yaw, pitch, roll => z, y, x
        self.rotation_matrix = Matrix4::from_angle_z(Rad(0.0));
        self.relative_rotate_euler(yaw, pitch, roll);

        self.build_matrix();
    }

    pub fn relative_rotate_euler(&mut self, yaw: Rad<f32>, pitch: Rad<f32>, roll: Rad<f32>) {
        // yaw, pitch, roll => z, y, x
        let new_rot =
            Matrix4::from_angle_z(yaw) * Matrix4::from_angle_y(pitch) * Matrix4::from_angle_x(roll);

        self.rotation_matrix = self.rotation_matrix * new_rot;

        self.build_matrix();
    }

    /// Retrieve the segments of the model
    pub fn get_segments(&self) -> &Vec<PbrModelSegment> {
        &self.segments
    }

    /// Retrieve the segments of the model
    ///
    /// Can be useful for changing a segment's material.
    ///
    /// # Example
    ///
    /// ```
    /// let segment = self.get_segments_mut().get_mut(0)?;
    ///
    /// // Modify the material that belongs to the segment
    /// segment.get_material_mut();
    /// ```
    pub fn get_segments_mut(&mut self) -> &mut Vec<PbrModelSegment> {
        &mut self.segments
    }

    /// Retrieve the position of the model
    pub fn get_position(&self) -> &Vector3<f32> {
        &self.position
    }

    /// Retrieve the rotation of the model
    //pub fn get_rotation(&self) -> &Vector3<Rad<f32>> {
    //&self.rotation
    //}

    fn is_close(x: f32, y: f32) -> bool {
        let rtol = 0.00001;
        let atol = 0.00000001;

        (x - y).abs() <= atol + rtol * y.abs()
    }

    /// Retrieve euler angles
    ///
    /// roll, pitch, yaw
    pub fn get_euler_angles(&self) -> Vector3<f32> {
        // psi, theta, phi => x, y, z => roll, pitch, yaw
        let mat = &self.rotation_matrix;
        let theta;
        let psi;
        let mut phi = 0.0;

        if Self::is_close(mat[2][0], -1.0) {
            theta = std::f32::consts::FRAC_PI_2;
            psi = mat[0][1].atan2(mat[0][2]);
        } else if Self::is_close(mat[2][0], 1.0) {
            theta = -std::f32::consts::FRAC_PI_2;
            psi = (-mat[0][1]).atan2(-mat[0][2]);
        } else {
            theta = -self.rotation_matrix[2][0].asin();
            let cos_theta = theta.cos();

            psi = (mat[2][1] / cos_theta).atan2(mat[2][2] / cos_theta);
            phi = (mat[1][0] / cos_theta).atan2(mat[0][0] / cos_theta);
        }

        [psi, theta, phi].into()
    }

    pub fn get_rotation_mat4(&self) -> &Matrix4<f32> {
        &self.rotation_matrix
    }

    pub fn get_rotation_mat3(&self) -> Matrix3<f32> {
        let mat4 = &self.rotation_matrix;
        let mat3 = [
            [mat4[0][0], mat4[0][1], mat4[0][2]],
            [mat4[1][0], mat4[1][1], mat4[1][2]],
            [mat4[2][0], mat4[2][1], mat4[2][2]],
        ];
        mat3.into()
    }

    pub fn get_basis3(&self) -> Basis3<f32> {
        let mat3 = self.get_rotation_mat3();
        let basis: Quaternion<f32> = mat3.into();
        let rot: Basis3<f32> = basis.into();

        rot
    }
}

impl DebugGUI for PbrModel {
    fn debug(&mut self, ui: &mut egui::Ui) {
        // Add location
        ui.label("position");
        let mut position: [f32; 3] = self.position.into();
        if DebugGUIFormat::position(ui, &mut position, -25.0..=25.0) {
            self.position = position.into();
            self.build_matrix();
        }

        // Add rotation
        ui.label("rotation");
        let mut rotation: [f32; 3] = (self.get_euler_angles()).into();

        //if DebugGUIFormat::rotation(ui, &mut rotation) {
        if DebugGUIFormat::euler(ui, &mut rotation) {
            self.set_rotation_euler(Rad(rotation[2]), Rad(rotation[1]), Rad(rotation[0]));
            self.build_matrix();
        }

        // Add segments
        for i in 0..self.segments.len() {
            let segment = &mut self.segments[i];

            egui::CollapsingHeader::new(format!("Segment {}", i)).show(ui, |ui| {
                segment.debug(ui);
            });
        }
    }
}
