use cgmath::Vector3;
use glium::backend::Facade;
use glium::index::IndicesSource;
use glium::vertex::VerticesSource;
use glium::{BackfaceCullingMode, DrawParameters, Frame, Program, Surface};
use std::any::Any;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use crate::renderer::SceneData;

use super::Material;

#[derive(Clone)]
pub struct Phong {
    light: Vector3<f32>,
    program: Arc<Program>,
}

impl Phong {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let mut vertex_shader_file = File::open("shaders/phong/vertex.glsl").unwrap();
        let mut vertex_shader_src = String::new();
        vertex_shader_file
            .read_to_string(&mut vertex_shader_src)
            .unwrap();
        let mut fragment_shader_file = File::open("shaders/phong/fragment.glsl").unwrap();
        let mut fragment_shader_src = String::new();
        fragment_shader_file
            .read_to_string(&mut fragment_shader_src)
            .unwrap();

        let program =
            Program::from_source(facade, &vertex_shader_src, &fragment_shader_src, None).unwrap();

        Self {
            light: [0.0; 3].into(),
            program: Arc::new(program),
        }
    }

    pub fn set_light_pos(&mut self, position: impl Into<Vector3<f32>>) {
        self.light = position.into();
    }
}

impl Material for Phong {
    fn render<'a>(
        &self,
        vertex_buffer: VerticesSource<'a>,
        index_buffer: IndicesSource<'a>,
        surface: &mut Frame,
        camera: [[f32; 4]; 4],
        position: [[f32; 4]; 4],
        _scene_data: &SceneData,
    ) {
        let light: [f32; 3] = self.light.clone().into();
        let uniforms = uniform! {
            u_light: light,
            projection: camera,
            view: position,
        };

        surface
            .draw(
                vertex_buffer,
                index_buffer,
                &*self.program,
                &uniforms,
                &DrawParameters {
                    backface_culling: BackfaceCullingMode::CullCounterClockwise,
                    depth: glium::Depth {
                        test: glium::DepthTest::IfLess,
                        write: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();
    }

    fn equal(&self, material: &dyn Any) -> bool {
        let simple = match material.downcast_ref::<Self>() {
            Some(simple) => simple,
            None => return false,
        };

        simple.light == self.light
    }

    fn to_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_material(&self) -> Box<dyn Material> {
        Box::new(self.clone())
    }
    fn clone_sized(&self) -> Self
    where
        Self: Sized,
    {
        self.clone()
    }
}

