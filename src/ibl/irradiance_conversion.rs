use crate::camera::Camera;
use crate::cubemap_loader::CubemapType;
use crate::cubemap_render::CubemapRender;
use glium::backend::Facade;
use glium::Program;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

pub struct IrradianceConverter {
    program: Arc<Program>,
}

impl IrradianceConverter {
    pub fn load(facade: &impl Facade) -> Self {
        let program = crate::material::insert_program!(
            "../shaders/irradiance_convolution/vertex.glsl",
            "../shaders/irradiance_convolution/fragment.glsl",
            facade
        );

        Self {
            program: Arc::new(program),
        }
    }

    pub fn calculate_to_fs<P>(
        &self,
        cubemap: &CubemapType,
        destination_dir: P,
        extension: &str,
        facade: &impl Facade,
        mut camera: Camera,
    ) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
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
        )?;

        Ok(())
    }
}
