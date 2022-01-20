use glium::{
    backend::Facade, index::IndicesSource, vertex::VerticesSource, BackfaceCullingMode,
    DrawParameters, Frame, Program, Surface,
};
use std::any::Any;
use std::sync::Arc;
use std::{fs::File, io::Read};

use super::Material;

#[derive(Clone)]
pub struct Simple {
    color: [f32; 3],
    program: Arc<Program>,
}

impl Simple {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let mut vertex_shader_file = File::open("shaders/simple/vertex.glsl").unwrap();
        let mut vertex_shader_src = String::new();
        vertex_shader_file
            .read_to_string(&mut vertex_shader_src)
            .unwrap();
        let mut fragment_shader_file = File::open("shaders/simple/fragment.glsl").unwrap();
        let mut fragment_shader_src = String::new();
        fragment_shader_file
            .read_to_string(&mut fragment_shader_src)
            .unwrap();

        let program =
            Program::from_source(facade, &vertex_shader_src, &fragment_shader_src, None).unwrap();

        Self {
            color: [1.0; 3],
            program: Arc::new(program),
        }
    }

    pub fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }
}

impl Material for Simple {
    fn render<'a>(
        &self,
        vertex_buffer: VerticesSource<'a>,
        index_buffer: IndicesSource<'a>,
        surface: &mut Frame,
        camera: [[f32; 4]; 4],
        position: [[f32; 4]; 4],
    ) {
        let uniforms = uniform! {
            material_color: self.color.clone(),
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

        simple.color == self.color
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