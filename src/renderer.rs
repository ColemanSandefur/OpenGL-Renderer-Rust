use cgmath::Rad;
use glium::framebuffer::SimpleFrameBuffer;
use glium::uniforms::Uniforms;
use glium::vertex::MultiVerticesSource;
use std::any::TypeId;
use std::collections::HashMap;

use crate::material::Material;
use crate::skybox::Skybox;
use cgmath::Matrix4;
use glium::index::IndicesSource;
use glium::{vertex::VerticesSource, Display};
use glium::{DrawError, DrawParameters, Frame, Program, Surface};

// Holds special information about the renderer
// I don't know if this will be needed, but it is here for future use if needed
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

// Holds the information needed to render an item
pub struct RenderEntry<'a> {
    vertex_buffer: VerticesSource<'a>,
    index_buffer: IndicesSource<'a>,
    material: &'a dyn Material,
}

impl<'a> RenderEntry<'a> {
    // world is a transformation matrix
    pub fn render(self, surface: &mut Renderable, scene: &SceneData, world: [[f32; 4]; 4]) {
        let camera = scene.camera;

        self.material.render(
            self.vertex_buffer,
            self.index_buffer,
            surface,
            camera,
            world,
            &scene,
        );
    }
}

// Passed to the renderer so that every shader can use the information
pub struct SceneData<'a> {
    camera: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    camera_rot: [Rad<f32>; 3],
    skybox: Option<&'a Skybox>,
}

impl<'a> SceneData<'a> {
    pub fn get_camera(&self) -> &[[f32; 4]; 4] {
        &self.camera
    }
    pub fn get_camera_pos(&self) -> &[f32; 3] {
        &self.camera_pos
    }
    pub fn get_camera_rot(&self) -> &[Rad<f32>; 3] {
        &self.camera_rot
    }
    pub fn get_skybox(&self) -> &Option<&'a Skybox> {
        &self.skybox
    }
}

impl<'a> Default for SceneData<'a> {
    fn default() -> Self {
        Self {
            camera: [[0.0; 4]; 4],
            camera_pos: [0.0; 3],
            camera_rot: [Rad(0.0); 3],
            skybox: None,
        }
    }
}

// Every frame will create a RenderScene, this will hold information like light sources,
// camera position and the skybox. When finish() is called it will render all the entries.
pub struct RenderScene<'a> {
    scene_data: SceneData<'a>,
    entries: HashMap<TypeId, Vec<RenderEntry<'a>>>,
}

impl<'a> RenderScene<'a> {
    // Add an item to be rendered
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

    // Used by renderer only
    fn new() -> Self {
        Self {
            scene_data: Default::default(),
            entries: HashMap::new(),
        }
    }

    pub fn set_camera(&mut self, camera: [[f32; 4]; 4]) {
        self.scene_data.camera = camera;
    }

    pub fn set_camera_pos(&mut self, pos: [f32; 3]) {
        self.scene_data.camera_pos = pos;
    }

    pub fn set_skybox(&mut self, skybox: Option<&'a Skybox>) {
        self.scene_data.skybox = skybox;
        if let Some(skybox) = skybox {
            skybox.render(self);
        }
    }

    pub fn get_skybox(&self) -> Option<&'a Skybox> {
        self.scene_data.skybox
    }

    pub fn get_scene_data(&self) -> &SceneData {
        &self.scene_data
    }

    // Render all the items
    pub fn finish(mut self, surface: &mut Renderable) {
        let skybox = match &self.scene_data.skybox {
            Some(skybox) => self.entries.remove(&skybox.get_skybox().as_any().type_id()),
            None => None,
        };

        let world: [[f32; 4]; 4] = (Matrix4::from_translation(self.scene_data.camera_pos.into())
            * Matrix4::from_angle_x(self.scene_data.camera_rot[0])
            * Matrix4::from_angle_y(self.scene_data.camera_rot[1])
            * Matrix4::from_angle_z(self.scene_data.camera_rot[2]))
        .into();

        for values in self.entries.into_values() {
            for entry in values {
                entry.render(surface, &self.scene_data, world);
            }
        }

        if let Some(skybox) = skybox {
            for entry in skybox {
                entry.render(surface, &self.scene_data, world);
            }
        }
    }
}

// Used for drawing with materials
// Need to add the other implementors but too lazy right now.
pub enum Renderable<'a> {
    Frame(&'a mut Frame),
    SimpleFrameBuffer(&'a mut SimpleFrameBuffer<'a>),
}

impl<'a> Renderable<'a> {
    pub fn draw<'b, 'c, V, I, U>(
        &mut self,
        vertex: V,
        index: I,
        program: &Program,
        uniforms: &U,
        draw_parameters: &DrawParameters<'_>,
    ) -> Result<(), DrawError>
    where
        V: MultiVerticesSource<'c>,
        I: Into<IndicesSource<'b>>,
        U: Uniforms,
    {
        match self {
            Self::Frame(frame) => frame.draw(vertex, index, program, uniforms, draw_parameters),
            Self::SimpleFrameBuffer(frame) => {
                frame.draw(vertex, index, program, uniforms, draw_parameters)
            }
        }
    }
}

impl<'a> From<&'a mut Frame> for Renderable<'a> {
    fn from(frame: &'a mut Frame) -> Self {
        Self::Frame(frame)
    }
}
impl<'a> From<&'a mut SimpleFrameBuffer<'a>> for Renderable<'a> {
    fn from(frame: &'a mut SimpleFrameBuffer<'a>) -> Self {
        Self::SimpleFrameBuffer(frame)
    }
}
