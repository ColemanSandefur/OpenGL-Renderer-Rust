use std::any::TypeId;
use std::collections::HashMap;

use crate::material::Material;
use cgmath::Matrix4;
use glium::index::IndicesSource;
use glium::Frame;
use glium::{vertex::VerticesSource, Display};

pub struct Renderer {
    display: Display,
}

impl Renderer {
    pub fn new(display: Display) -> Self {
        Self { display }
    }

    pub fn begin_scene(&self) -> RenderScene {
        RenderScene::new()
    }
}

pub struct RenderEntry<'a> {
    vertex_buffer: VerticesSource<'a>,
    index_buffer: IndicesSource<'a>,
    material: &'a dyn Material,
}

impl<'a> RenderEntry<'a> {
    pub fn render(self, surface: &mut Frame, camera: [[f32; 4]; 4], camera_pos: [f32; 3]) {
        let world: [[f32; 4]; 4] = Matrix4::from_translation(camera_pos.into()).into();

        self.material.render(
            self.vertex_buffer,
            self.index_buffer,
            surface,
            camera,
            world,
        );
    }
}

pub struct RenderScene<'a> {
    camera: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    entries: HashMap<TypeId, Vec<RenderEntry<'a>>>,
}

impl<'a> RenderScene<'a> {
    pub fn publish<V, I>(&mut self, vertex_buffer: V, index_buffer: I, material: &'a dyn Material)
    where
        V: Into<VerticesSource<'a>>,
        I: Into<IndicesSource<'a>>,
    {
        let entry = RenderEntry {
            vertex_buffer: vertex_buffer.into(),
            index_buffer: index_buffer.into(),
            material,
        };

        let type_id = material.as_any().type_id();

        if !self.entries.contains_key(&type_id) {
            self.entries.insert(type_id, Vec::new());
        }

        self.entries.get_mut(&type_id).unwrap().push(entry);
    }

    pub fn new() -> Self {
        Self {
            camera: [[0.0; 4]; 4],
            camera_pos: [0.0; 3],
            entries: HashMap::new(),
        }
    }

    pub fn set_camera(&mut self, camera: [[f32; 4]; 4]) {
        self.camera = camera;
    }

    pub fn set_camera_pos(&mut self, pos: [f32; 3]) {
        self.camera_pos = pos;
    }

    pub fn finish(self, surface: &mut Frame) {
        for values in self.entries.into_values() {
            for entry in values {
                entry.render(surface, self.camera.clone(), self.camera_pos.clone());
            }
        }
    }
}
