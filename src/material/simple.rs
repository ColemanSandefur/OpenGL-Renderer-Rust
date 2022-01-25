use glium::{
    backend::Facade, index::IndicesSource, vertex::VerticesSource, BackfaceCullingMode,
    DrawParameters, Program,
};
use std::any::Any;
use std::sync::Arc;

use crate::renderer::{Renderable, SceneData};

use super::Material;

#[derive(Clone)]
pub struct Simple {
    color: [f32; 3],
    program: Arc<Program>,
}

impl Simple {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let program =
            crate::material::load_program(facade, "shaders/simple/".into());

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
        surface: &mut Renderable,
        camera: [[f32; 4]; 4],
        position: [[f32; 4]; 4],
        _scene_data: &SceneData,
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
