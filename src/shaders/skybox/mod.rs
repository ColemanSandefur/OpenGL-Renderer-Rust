use crate::insert_program;
use crate::shader::Shader;
use crate::utils::pbr_skybox::PBRSkybox;
use glium::backend::Facade;
use glium::DrawParameters;
use glium::Program;
use nalgebra::Matrix4;
use std::any::Any;
use std::rc::Rc;

#[derive(Clone)]
pub struct Skybox {
    program: Rc<Program>,
}

impl Skybox {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let program = Rc::new(insert_program!("./vertex.glsl", "./fragment.glsl", facade));

        Self { program }
    }
}

impl Shader for Skybox {
    fn render<'a>(
        &self,
        vertex_buffer: glium::vertex::VerticesSource<'a>,
        index_buffer: glium::index::IndicesSource<'a>,
        surface: &mut crate::renderer::Renderable,
        camera: [[f32; 4]; 4],
        position: [[f32; 4]; 4],
        scene_data: &crate::renderer::SceneData,
    ) {
        let cubemap = scene_data
            .get_scene_object::<PBRSkybox>()
            .expect("no skybox provided")
            .get_skybox();

        let uniforms = uniform! {
            projection: camera,
            view: position,
            environmentMap: &**cubemap
        };

        surface
            .draw(
                vertex_buffer,
                index_buffer,
                &self.program,
                &uniforms,
                &DrawParameters {
                    depth: glium::Depth {
                        test: glium::DepthTest::IfLessOrEqual,
                        write: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();
    }

    fn get_model_mat(&self) -> Matrix4<f32> {
        Matrix4::zeros()
    }

    fn set_model_mat(&mut self, _model: Matrix4<f32>) {}

    fn equal_shader(&self, _shader: &dyn std::any::Any) -> bool {
        false
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

    fn clone_shader(&self) -> Box<dyn Shader> {
        Box::new(self.clone())
    }
    fn clone_sized(&self) -> Self {
        self.clone()
    }
}
