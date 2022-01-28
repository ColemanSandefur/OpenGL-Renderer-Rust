use glium::{backend::Facade, Program};
use std::path::PathBuf;
use std::sync::Arc;

use crate::camera::Camera;
use crate::cubemap_loader::CubemapType;
use crate::cubemap_render::CubemapRender;

pub struct Prefilter {
    program: Arc<Program>,
}

impl Prefilter {
    const MAX_MIP_LEVELS: u32 = 5;
    pub fn load(facade: &impl Facade) -> Self {
        let program = crate::material::load_program(facade, "./shaders/prefilter/".into());

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
        camera: Camera,
    ) {
        let output_size = (128, 128);

        let cubemap_render = CubemapRender::new(facade);

        let cubemap_mip = match cubemap {
            CubemapType::Cubemap(c) => c.get_mipmap_levels(),
            CubemapType::SrgbCubemap(c) => c.get_mipmap_levels(),
        };

        let mip_levels = cubemap_mip.min(Self::MAX_MIP_LEVELS);

        for level in 0..mip_levels as i32 {
            let output_size = ((output_size.0 as f32 * (0.5f32).powi(level)) as u32, (output_size.1 as f32 * (0.5f32).powi(level)) as u32);
            let generate_uniforms = |projection, view| {
                uniform! {
                    environment_map: cubemap,
                    projection: projection,
                    view: view,
                    roughness: level as f32 / mip_levels as f32,
                }
            };
            cubemap_render.render(
                output_size,
                destination_dir.join(format!("{}", level)),
                extension,
                facade,
                camera,
                generate_uniforms,
                &*self.program,
            );

        }
    }
}
