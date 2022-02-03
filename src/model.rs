use crate::renderer::RenderScene;
use std::error::Error;
use russimp::scene::PostProcess;
use russimp::scene::Scene;
use cgmath::Matrix4;
use cgmath::Rad;
use cgmath::Vector3;
use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use std::path::PathBuf;

use crate::{material::Material, vertex::Vertex};

/// Section of a [`Model`]
///
/// Models often consist of multiple smaller models, I am calling them segments.
pub struct ModelSegment<T: Material> {
    material: T,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u32>,
}
impl<T: Material> ModelSegment<T> {
    pub fn new(
        vertex_buffer: VertexBuffer<Vertex>,
        index_buffer: IndexBuffer<u32>,
        material: T,
    ) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            material,
        }
    }
    pub fn build_matrix(&mut self, model: Matrix4<f32>) {
        for vert in &mut *self.vertex_buffer.map() {
            vert.model = model.into();
        }
    }

    pub fn render<'a>(&'a self, scene: &mut RenderScene<'a>) {
        scene.publish(&self.vertex_buffer, &self.index_buffer, &self.material);
    }

    pub fn get_material(&self) -> &T {
        &self.material
    }
    pub fn get_material_mut(&mut self) -> &mut T {
        &mut self.material
    }
}

/// A simple model container
///
/// The most basic form of a model provided in this library. It can have any type of [`Material`].
/// The downside is that models loaded from the file system won't have the right materials. You
/// should use a wrapper struct like [`PbrModel`] to have materials loaded from the file system.
/// The `Model` also holds the position and rotation of the model (in world space). This is 
/// how you will move the model around in the scene. To render the model you will call `render`
/// on the object and pass in a [`RenderScene`] which will control when the model should be rendered.
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
///
/// [`PbrModel`]: crate::pbr_model::PbrModel

pub struct Model<T: Material> {
    position: Vector3<f32>,
    rotation: Vector3<Rad<f32>>,
    segments: Vec<ModelSegment<T>>,
}

impl<T: Material> Model<T> {
    /// Loads a model file from the file system
    ///
    /// Can be used for multiple types of models, I have only tested Wavefront (.obj) or glTF 2.0
    /// (.glb). This won't set/load any materials from the file (due to the generics), but the vertices and normals
    /// should be right.
    pub fn load_from_fs(path: PathBuf, facade: &impl Facade, material: T) -> Result<Self, Box<dyn Error>> {
        let scene = Scene::from_file(
            path.as_os_str().to_str().ok_or("file path couldn't be made into a string")?,
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
            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            for index in 0..mesh.vertices.len() {
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

                vertices.push(Vertex {
                    position,
                    normal,
                    tex_coords,
                    ..Default::default()
                });
            }

            for face in mesh.faces {
                for index in face.0 {
                    indices.push(index);
                }
            }

            let index_buffer =
                IndexBuffer::new(facade, glium::index::PrimitiveType::TrianglesList, &indices)?;
            let vertex_buffer = VertexBuffer::new(facade, &vertices)?;

            let material = material.clone_sized();

            segments.push(ModelSegment::new(vertex_buffer, index_buffer, material));
        }

        Ok(Self {
            position: [0.0; 3].into(),
            rotation: [Rad(0.0); 3].into(),
            segments,
        })
    }

    /// Submits the PbrModel to the [`RenderScene`] to be rendered
    ///
    /// Passes the model to the [`RenderScene`]. The [`RenderScene`]
    /// holds this struct by reference, so this struct must outlive the [`RenderScene`].
    ///
    /// [`RenderScene`]: crate::renderer::RenderScene
    pub fn render<'a>(&'a self, scene: &mut RenderScene<'a>) {
        //scene.publish(&self.vertex_buffer, &self.index_buffer, &self.material);
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
        let rotation_mat = Matrix4::from_angle_x(self.rotation.x)
            * Matrix4::from_angle_y(self.rotation.y)
            * Matrix4::from_angle_z(self.rotation.z);
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
        self.rotation[0] += rotation[0];
        self.rotation[1] += rotation[1];
        self.rotation[2] += rotation[2];
        self.build_matrix();
    }

    /// Retrieve the segments of the model
    pub fn get_segments(&self) -> &Vec<ModelSegment<T>> {
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
    pub fn get_segments_mut(&mut self) -> &mut Vec<ModelSegment<T>> {
        &mut self.segments
    }
}
