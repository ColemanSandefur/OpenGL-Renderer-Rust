use glium::index::IndicesSource;
use glium::uniforms::Uniforms;
use glium::vertex::MultiVerticesSource;
use glium::DrawError;
use glium::DrawParameters;
use glium::Program;
use glium::Surface;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;

use glium::vertex::VerticesSource;
use glium::{framebuffer::SimpleFrameBuffer, Frame};

use crate::shader::Shader;
use crate::utils::camera::Camera;

pub struct Renderer {
    polygons: u32,
}

impl Renderer {
    pub fn new() -> Self {
        Self { polygons: 0 }
    }

    pub fn begin_scene(&mut self) -> RenderScene {
        return RenderScene::new(self);
    }
    pub fn get_polygons(&self) -> u32 {
        self.polygons
    }
}

pub struct RenderEntry<'a> {
    vertex_buffer: VerticesSource<'a>,
    index_buffer: IndicesSource<'a>,
    material: &'a dyn Shader,
}

impl<'a> RenderEntry<'a> {
    pub fn render(
        self,
        surface: &mut Renderable,
        scene: &SceneData,
        world: impl Into<[[f32; 4]; 4]>,
    ) {
        self.material.render(
            self.vertex_buffer,
            self.index_buffer,
            surface,
            scene.projection,
            world.into(),
            &scene,
        );
    }
}

pub struct SceneData {
    pub projection: [[f32; 4]; 4],
    pub camera: Camera,
    scene_objects: HashMap<TypeId, Box<dyn Any>>,
    scene_vars: HashMap<&'static str, Box<dyn Any>>,
}

enum SceneObject<'a> {
    Owned(Box<dyn Any>),
    Borrowed(&'a dyn Any),
    BorrowedMut(&'a mut dyn Any),
}

impl SceneData {
    fn new() -> Self {
        Self {
            projection: [[0.0; 4]; 4],
            camera: Camera::new(),
            scene_objects: HashMap::new(),
            scene_vars: HashMap::new(),
        }
    }
    pub fn get_scene_object_raw<T: 'static + Sized>(&self) -> Option<&Box<dyn Any>> {
        self.scene_objects.get(&TypeId::of::<T>())
    }
    pub fn get_scene_object_raw_mut<T: 'static + Sized>(&mut self) -> Option<&mut Box<dyn Any>> {
        self.scene_objects.get_mut(&TypeId::of::<T>())
    }

    pub fn get_scene_object<T: 'static + Sized>(&self) -> Option<&T> {
        let object = self.get_scene_object_raw::<T>()?;

        object.downcast_ref()
    }

    pub fn get_scene_object_mut<T: 'static + Sized>(&mut self) -> Option<&mut T> {
        let object = self.get_scene_object_raw_mut::<T>()?;

        object.downcast_mut()
    }

    pub fn set_scene_object<T: Any>(&mut self, data: T) {
        self.scene_objects.insert(TypeId::of::<T>(), Box::new(data));
    }
}

pub struct RenderScene<'a> {
    pub scene_data: SceneData,
    entries: HashMap<TypeId, Vec<RenderEntry<'a>>>,
    renderer: &'a mut Renderer,
}

impl<'a> RenderScene<'a> {
    fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            scene_data: SceneData::new(),
            entries: HashMap::new(),
            renderer,
        }
    }

    pub fn publish<V, I>(&mut self, vertex_buffer: V, index_buffer: I, shader: &'a dyn Shader)
    where
        V: Into<VerticesSource<'a>>,
        I: Into<IndicesSource<'a>>,
    {
        let entry = RenderEntry {
            vertex_buffer: vertex_buffer.into(),
            index_buffer: index_buffer.into(),
            material: shader,
        };

        let type_id = shader.as_any().type_id();

        self.entries.entry(type_id).or_insert(Vec::new());

        self.entries.get_mut(&type_id).unwrap().push(entry);
    }

    /// Render all the items that have been submitted
    pub fn finish(mut self, surface: &mut Renderable) {
        //let skybox = match &self.scene_data.skybox {
        //Some(skybox) => self.entries.remove(&skybox.get_skybox().as_any().type_id()),
        //None => None,
        //};

        let world: [[f32; 4]; 4] = self.scene_data.camera.get_view_matrix().into();

        //if let Some(skybox) = skybox {
        //for entry in skybox {
        //entry.render(surface, &self.scene_data, world);
        //}
        //}

        let mut vertices = 0;
        for values in self.entries.into_values() {
            for entry in values {
                // Crudely count indices
                vertices += match &entry.index_buffer {
                    IndicesSource::IndexBuffer { buffer, .. } => buffer.get_elements_count(),
                    IndicesSource::MultidrawArray { buffer, .. } => buffer.get_elements_count(),
                    _ => 0,
                };
                entry.render(surface, &self.scene_data, world);
            }
        }

        // Assume that each polygon is a triangle (vertices / 3)
        self.renderer.polygons = vertices as u32 / 3;
    }
}

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
