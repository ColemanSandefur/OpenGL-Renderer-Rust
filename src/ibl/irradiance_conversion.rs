use crate::camera::Camera;
use crate::cubemap_loader::CubemapType;
use crate::cubemap_render::CubemapRender;
use glium::backend::Facade;
use glium::Program;
use std::path::PathBuf;
use std::sync::Arc;

pub struct IrradianceConverter {
    program: Arc<Program>,
}

impl IrradianceConverter {
    pub fn load(facade: &impl Facade) -> Self {
        let program =
            crate::material::load_program(facade, "shaders/irradiance_convolution".into());

        Self {
            program: Arc::new(program),
        }
    }

    pub fn calculate_to_fs(
        &self,
        cubemap: &CubemapType,
        destination_dir: PathBuf,
        extension: &str,
        facade: &impl Facade,
        mut camera: Camera,
    ) {
        let output_size = (32, 32);
        camera.set_width(output_size.0);
        camera.set_height(output_size.1);
        let generate_uniforms = |projection, view| {
            uniform! {
                environment_map: cubemap,
                projection: projection,
                view: view,
            }
        };

        let cubemap_render = CubemapRender::new(facade);
        cubemap_render.render(
            output_size,
            destination_dir,
            extension,
            facade,
            camera,
            generate_uniforms,
            &*self.program,
        );
    }
}
