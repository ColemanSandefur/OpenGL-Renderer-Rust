use cgmath::Rad;
use cgmath::Vector3;
use glium::framebuffer::SimpleFrameBuffer;
use glium::uniforms::Uniforms;
use glium::vertex::MultiVerticesSource;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;

use crate::lights::RawLights;
use crate::material::Material;
use crate::skybox::Skybox;
use cgmath::Matrix4;
use glium::index::IndicesSource;
use glium::{vertex::VerticesSource, Display};
use glium::{DrawError, DrawParameters, Frame, Program, Surface};

/// Main renderer of the program
///
/// Currently its only use is to generate a [`RenderScene`] which renders a single frame.
// Holds special information about the renderer
// I don't know if this will be needed, but it is here for future use if needed
pub struct Renderer {
    _display: Display,
    polygons: u32,
}

impl Renderer {
    pub fn new(display: Display) -> Self {
        Self {
            _display: display,
            polygons: 0,
        }
    }

    pub fn begin_scene(&mut self) -> RenderScene {
        RenderScene::new(self)
    }

    pub fn get_polygons(&self) -> u32 {
        self.polygons
    }
}

/// Holds the information needed to render an item
///
/// When an item is submitted to the [`RenderScene`], a `RenderEntry` is created which holds the
/// vertex and index buffers, and the material that will be used to render the model.
pub struct RenderEntry<'a> {
    vertex_buffer: VerticesSource<'a>,
    index_buffer: IndicesSource<'a>,
    material: &'a dyn Material,
}

impl<'a> RenderEntry<'a> {
    /// Render the buffers
    ///
    /// Uses the index and vertex buffers along with the material to render to the surface.
    ///
    /// World is a pre-computed transformation matrix
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

/// Scene specific data
///
/// Contains information that should belong to the scene, such as the camera position, and skybox.
/// It is also passed to the [`Material`](crate::material::Material) during rendering so that every shader can use the information
pub struct SceneData<'a> {
    camera: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    camera_rot: [Rad<f32>; 3],
    skybox: Option<&'a Skybox>,
    raw_lights: RawLights,
    scene_variables: HashMap<TypeId, Box<dyn Any>>,
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

    /// Gets scene variable
    pub fn get_scene_variable_raw<T: 'static + ?Sized>(&self) -> Option<&Box<dyn Any>> {
        self.scene_variables.get(&TypeId::of::<T>())
    }

    /// Gets mut scene variable
    pub fn get_scene_variable_mut_raw<T: 'static + ?Sized>(&mut self) -> Option<&mut Box<dyn Any>> {
        self.scene_variables.get_mut(&TypeId::of::<T>())
    }

    /// Gets scene variable and auto downcasts it.
    pub fn get_scene_variable<T: 'static + Sized>(&self) -> Option<&T> {
        let data = self.get_scene_variable_raw::<T>()?;

        data.downcast_ref()
    }

    pub fn add_scene_variable<T: Any>(&mut self, data: T) {
        self.scene_variables
            .insert(TypeId::of::<T>(), Box::new(data));
    }

    /// Gets mutable scene variable and auto downcasts it.
    pub fn get_scene_variable_mut<T: 'static + Sized>(&mut self) -> Option<&mut T> {
        let data = self.get_scene_variable_mut_raw::<T>()?;

        data.downcast_mut()
    }

    pub fn get_raw_lights(&self) -> &RawLights {
        &self.raw_lights
    }

    pub fn get_raw_lights_mut(&mut self) -> &mut RawLights {
        &mut self.raw_lights
    }
}

impl<'a> Default for SceneData<'a> {
    fn default() -> Self {
        Self {
            camera: [[0.0; 4]; 4],
            camera_pos: [0.0; 3],
            camera_rot: [Rad(0.0); 3],
            skybox: None,
            raw_lights: RawLights::new(),
            scene_variables: HashMap::new(),
        }
    }
}

/// A struct responsible for rendering a single frame
///
/// When you want to render a frame, you must first generate a `RenderScene` from [`Renderer`].
/// Then you set the scene related variables (most of which are found at [`SceneData`]). Then you
/// can publish a model to be rendered by calling `publish` on the `RenderScene`. When you finish
/// giving the scene data, you call `finish` and the `RenderScene` will be consumed and render
/// everything that was published to it.
pub struct RenderScene<'a> {
    scene_data: SceneData<'a>,
    entries: HashMap<TypeId, Vec<RenderEntry<'a>>>,
    renderer: &'a mut Renderer,
}

pub struct Test {}
impl<'a> RenderScene<'a> {
    /// Add an item to be rendered
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

        self.entries.entry(type_id).or_insert(Vec::new());

        self.entries.get_mut(&type_id).unwrap().push(entry);
    }

    /// Will check if the object published is in the camera's fov
    ///
    /// Currently identical to publish as I haven't implemented frustum culling yet
    pub fn publish_bounding<V, I>(
        &mut self,
        vertex_buffer: V,
        index_buffer: I,
        _bounds: (Vector3<f32>, Vector3<f32>),
        material: &'a dyn Material,
    ) where
        V: Into<VerticesSource<'a>>,
        I: Into<IndicesSource<'a>>,
    {
        let entry = RenderEntry {
            vertex_buffer: vertex_buffer.into(),
            index_buffer: index_buffer.into(),
            material,
        };

        let type_id = material.as_any().type_id();

        self.entries.entry(type_id).or_insert(Vec::new());

        self.entries.get_mut(&type_id).unwrap().push(entry);
    }

    /// Used by renderer only
    fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            scene_data: Default::default(),
            entries: HashMap::new(),
            renderer,
        }
    }

    pub fn set_camera(&mut self, camera: [[f32; 4]; 4]) {
        self.scene_data.camera = camera;
    }

    pub fn set_camera_pos(&mut self, pos: [f32; 3]) {
        self.scene_data.camera_pos = pos;
    }
    pub fn set_camera_rot(&mut self, rot: [Rad<f32>; 3]) {
        self.scene_data.camera_rot = rot;
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

    pub fn get_scene_data_mut(&mut self) -> &mut SceneData<'a> {
        &mut self.scene_data
    }

    /// Render all the items that have been submitted
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

        let mut faces = 0;

        if let Some(skybox) = skybox {
            for entry in skybox {
                entry.render(surface, &self.scene_data, world);
            }
        }

        for values in self.entries.into_values() {
            for entry in values {
                // Crudely count indices
                faces += match &entry.index_buffer {
                    IndicesSource::IndexBuffer { buffer, .. } => buffer.get_elements_count(),
                    IndicesSource::MultidrawArray { buffer, .. } => buffer.get_elements_count(),
                    _ => 0,
                };
                entry.render(surface, &self.scene_data, world);
            }
        }

        // Assume that each polygon is a triangle (vertices / 3)
        self.renderer.polygons = faces as u32 / 3;
    }
}

/// Used for drawing with [`Material`]
///
/// Since [`Material`] is a trait that needs to be made into an object, it cannot have generics. I
/// made this `Renderable` enum to have a way to render to a frame, or a frame buffer.
///
/// I Need to add the other implementors but too lazy right now.
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
