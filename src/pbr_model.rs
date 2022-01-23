use crate::material::PBR;
use crate::renderer::RenderScene;
use cgmath::Matrix4;
use cgmath::Rad;
use cgmath::Vector3;
use glium::backend::Facade;
use glium::{IndexBuffer, VertexBuffer};
use russimp::material::PropertyTypeInfo::FloatArray;
use russimp::scene::PostProcess;
use russimp::scene::Scene;
use std::path::PathBuf;
use tobj::LoadOptions;

use crate::vertex::Vertex;

// Model that uses the Physically Based Rendering shader

// Models often consist of multiple smaller models, I am calling them segments
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

    pub fn build_matrix(&mut self, model: Matrix4<f32>) {
        for vert in &mut *self.vertex_buffer.map() {
            vert.model = model.into();
        }
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
}

// The main model, this will control things that affect the whole model. You can also gain access
// to the segments of the model if you wish.
pub struct PbrModel {
    material: PBR,
    position: Vector3<f32>,
    rotation: Vector3<Rad<f32>>,
    segments: Vec<PbrModelSegment>,
}

impl PbrModel {
    // Loads a glTF 2.0 (.glb) file
    pub fn load_from_fs2(path: PathBuf, facade: &impl Facade, material: PBR) -> Self {
        let scene = Scene::from_file(
            path.as_os_str().to_str().unwrap(),
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
        )
        .unwrap();

        let mut segments = Vec::new();

        for mesh in scene.meshes {
            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            for index in 0..mesh.vertices.len() {
                let vertex = mesh.vertices[index as usize];
                let position: [f32; 3] = [vertex.x, vertex.y, vertex.z];
                let normal_vec = mesh.normals[index as usize];
                let normal = [normal_vec.x, normal_vec.y, normal_vec.z];
                vertices.push(Vertex {
                    position,
                    normal,
                    ..Default::default()
                });
            }

            for face in mesh.faces {
                for index in face.0 {
                    indices.push(index);
                }
            }

            let index_buffer =
                IndexBuffer::new(facade, glium::index::PrimitiveType::TrianglesList, &indices)
                    .unwrap();
            let vertex_buffer = VertexBuffer::new(facade, &vertices).unwrap();

            let mut material = material.clone();

            let scene_material = &scene.materials[mesh.material_index as usize];
            for property in &scene_material.properties {
                if property.key == "$clr.base" {
                    if let FloatArray(data) = &property.data {
                        material.get_pbr_params_mut().albedo = [data[0], data[1], data[2]].into();
                    }
                }

                if property.key == "$mat.roughnessFactor" {
                    if let FloatArray(data) = &property.data {
                        material.get_pbr_params_mut().roughness = data[0];
                    }
                }

                if property.key == "$mat.metallicFactor" {
                    if let FloatArray(data) = &property.data {
                        material.get_pbr_params_mut().metallic = data[0];
                    }
                }
            }

            segments.push(PbrModelSegment::new(vertex_buffer, index_buffer, material));
        }

        Self {
            material,
            position: [0.0; 3].into(),
            rotation: [Rad(0.0); 3].into(),
            segments,
        }
    }

    // Loads a Wavefront (.obj) file
    pub fn load_from_fs(path: PathBuf, facade: &impl Facade, material: PBR) -> Self {
        let (models, materials) = tobj::load_obj(
            path.as_os_str().to_str().unwrap(),
            &LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )
        .unwrap();

        if materials.is_ok() {
            for material in materials.as_ref().unwrap() {
                println!("{:#?}", material);
            }
        }

        let mut segments = Vec::new();

        for model in models {
            println!("num texcoords {}", model.mesh.texcoords.len() / 2);
            //println!("{}", model.name);
            let mut vertices: Vec<Vertex> = Vec::new();
            let indices: Vec<u32> = model.mesh.indices;

            let num_vertices = model.mesh.positions.len() / 3;

            // Load x, y, z, positions for all vertices
            for triplet in 0..num_vertices {
                let index = triplet * 3;
                let x = model.mesh.positions[index];
                let y = model.mesh.positions[index + 1];
                let z = model.mesh.positions[index + 2];

                vertices.push(Vertex {
                    position: [x, y, z],
                    ..Default::default()
                });
            }

            // Load the normals for all veritces
            for triplet in 0..num_vertices {
                let index = triplet * 3;
                if model.mesh.normals.get(index).is_none() {
                    break;
                }
                let x = model.mesh.normals[index];
                let y = model.mesh.normals[index + 1];
                let z = model.mesh.normals[index + 2];

                match vertices.get_mut(triplet) {
                    Some(vertex) => {
                        vertex.normal = [x, y, z];
                    }
                    None => {
                        println!("vertex {} is missing", index);
                    }
                };
            }

            let index_buffer =
                IndexBuffer::new(facade, glium::index::PrimitiveType::TrianglesList, &indices)
                    .unwrap();
            let vertex_buffer = VertexBuffer::new(facade, &vertices).unwrap();

            let mut material = material.clone();

            if let Some(material_index) = model.mesh.material_id {
                let given_material = materials.as_ref().unwrap().get(material_index).unwrap();
                material.get_pbr_params_mut().albedo = given_material.diffuse.into();
            }

            segments.push(PbrModelSegment::new(vertex_buffer, index_buffer, material));
        }

        Self {
            material,
            position: [0.0; 3].into(),
            rotation: [Rad(0.0); 3].into(),
            segments,
        }
    }

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

    pub fn render<'a>(&'a self, scene: &mut RenderScene<'a>) {
        //scene.publish(&self.vertex_buffer, &self.index_buffer, &self.material);
        for item in &self.segments {
            item.render(scene);
        }
    }

    pub fn relative_move(&mut self, position: impl Into<Vector3<f32>>) {
        self.position = self.position + position.into();
        self.build_matrix();
    }

    pub fn relative_rotate(&mut self, rotation: impl Into<Vector3<Rad<f32>>>) {
        let rotation = rotation.into();
        self.rotation[0] += rotation[0];
        self.rotation[1] += rotation[1];
        self.rotation[2] += rotation[2];
        self.build_matrix();
    }
    pub fn get_material(&self) -> &PBR {
        &self.material
    }
    pub fn get_material_mut(&mut self) -> &mut PBR {
        &mut self.material
    }

    pub fn get_segments(&self) -> &Vec<PbrModelSegment> {
        &self.segments
    }
    pub fn get_segments_mut(&mut self) -> &mut Vec<PbrModelSegment> {
        &mut self.segments
    }
}
