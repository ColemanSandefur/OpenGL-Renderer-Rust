use crate::renderer::Renderable;
use cgmath::Vector3;
use glium::backend::Facade;
use glium::index::IndicesSource;
use glium::vertex::VerticesSource;
use glium::{BackfaceCullingMode, DrawParameters, Program};
use std::any::Any;
use std::sync::Arc;

use crate::renderer::SceneData;

use super::Material;

#[derive(Clone, Copy)]
pub struct MaterialParams {
    pub ambient: Vector3<f32>,
    pub diffuse: Vector3<f32>,
    pub specular: Vector3<f32>,
    pub shininess: f32,
}

#[derive(Clone, Copy)]
pub struct MaterialParamsOutput {
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
}

impl From<MaterialParams> for MaterialParamsOutput {
    fn from(from: MaterialParams) -> Self {
        Self {
            ambient: from.ambient.into(),
            diffuse: from.diffuse.into(),
            specular: from.specular.into(),
            shininess: from.shininess,
        }
    }
}

impl Default for MaterialParams {
    fn default() -> Self {
        Self {
            ambient: [1.0, 0.5, 0.31].into(),
            diffuse: [1.0, 0.5, 0.31].into(),
            specular: [0.5, 0.5, 0.5].into(),
            shininess: 32.0,
        }
    }
}

/// Just a basic shader.
///
/// This shader renders objects and basic lighting.
#[derive(Clone)]
pub struct Basic {
    light_pos: Vector3<f32>,
    light_color: Vector3<f32>,
    program: Arc<Program>,
    basic_params: MaterialParams,
}

impl Basic {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let program = crate::material::insert_program!("../shaders/basic/vertex.glsl", "../shaders/basic/fragment.glsl", facade);

        Self {
            light_pos: [0.0; 3].into(),
            light_color: [1.0; 3].into(),
            program: Arc::new(program),
            basic_params: MaterialParams {
                ..Default::default()
            },
        }
    }

    pub fn set_material_params(&mut self, params: MaterialParams) {
        self.basic_params = params;
    }
    pub fn get_material_params(&self) -> &MaterialParams {
        &self.basic_params
    }
    pub fn get_material_params_mut(&mut self) -> &mut MaterialParams {
        &mut self.basic_params
    }

    pub fn set_light_pos(&mut self, pos: impl Into<Vector3<f32>>) {
        self.light_pos = pos.into();
    }
    pub fn set_light_color(&mut self, color: impl Into<Vector3<f32>>) {
        self.light_color = color.into();
    }
}

impl Material for Basic {
    fn render<'a>(
        &self,
        vertex_buffer: VerticesSource<'a>,
        index_buffer: IndicesSource<'a>,
        surface: &mut Renderable,
        camera: [[f32; 4]; 4],
        position: [[f32; 4]; 4],
        _scene_data: &SceneData,
    ) {
        let light_pos: [f32; 3] = self.light_pos.clone().into();
        let light_color: [f32; 3] = self.light_color.clone().into();
        let camera_pos: [f32; 3] = [position[3][0], position[3][1], position[3][2]];

        let ambient: [f32; 3] = self.basic_params.ambient.into();
        let diffuse: [f32; 3] = self.basic_params.diffuse.into();
        let specular: [f32; 3] = self.basic_params.specular.into();
        let shininess = self.basic_params.shininess;

        let uniforms = uniform! {
            light_pos: light_pos,
            light_color: light_color,
            projection: camera,
            view: position,
            camera_pos: camera_pos,
            ambient: ambient,
            diffuse: diffuse,
            specular: specular,
            shininess: shininess,
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

        simple.light_pos == self.light_pos && simple.light_color == self.light_color
        //&& self.pbr_params == simple.pbr_params
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
