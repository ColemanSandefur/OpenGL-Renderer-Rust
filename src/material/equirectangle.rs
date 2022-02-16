use crate::camera::Camera;
use crate::cubemap_render::CubemapRender;
use glium::backend::Facade;
use glium::texture::RawImage2d;
use glium::Program;
use glium::Texture2d;
use image::hdr::HdrDecoder;
use image::io::Reader as ImageReader;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

/// Converts rectangular map into cubemap.
///
/// Most HDR environment maps consist of a sphere projected onto a rectangle, this material is
/// designed to convert this equirectangle to a cubemap.
#[derive(Clone)]
pub struct Equirectangle {
    program: Arc<Program>,
}

impl Equirectangle {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let program = crate::material::insert_program!("../shaders/equirectangle_to_cube/vertex.glsl", "../shaders/equirectangle_to_cube/fragment.glsl", facade);

        Self {
            program: Arc::new(program),
        }
    }

    pub fn compute_from_fs(
        &self,
        source: PathBuf,
        destination_dir: PathBuf,
        extension: &str,
        facade: &impl Facade,
        camera: Camera,
    ) {
        let output_size = (1024, 1024);

        let (source_data, source_dimensions) = {
            let image = ImageReader::open(source)
                .unwrap()
                .decode()
                .unwrap()
                .into_rgb8();

            let dimensions = image.dimensions();

            (image.into_raw(), dimensions)
        };
        let source_image = RawImage2d::from_raw_rgb(source_data, source_dimensions);

        let source_texture = Texture2d::new(facade, source_image).unwrap();

        let generate_uniforms = |projection, view| {
            uniform! {
                equirectangular_map: &source_texture,
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
    pub fn compute_from_fs_hdr(
        &self,
        source: PathBuf,
        destination_dir: PathBuf,
        extension: &str,
        facade: &impl Facade,
        camera: Camera,
    ) {
        let output_size = (1024, 1024);

        let (source_data, source_dimensions) = {
            let buffer = BufReader::new(File::open(source).unwrap());
            let hdr_image = HdrDecoder::new(buffer).unwrap();
            let dimensions = (hdr_image.metadata().width, hdr_image.metadata().height);

            let output: Vec<f32> = hdr_image
                .read_image_hdr()
                .unwrap()
                .into_iter()
                .flat_map(|rgb| {
                    return {
                        let values = rgb.0;
                        values
                    };
                })
                .collect();

            (output, dimensions)
        };

        let source_image = RawImage2d::from_raw_rgb(source_data, source_dimensions);

        let source_texture = Texture2d::new(facade, source_image).unwrap();

        let generate_uniforms = |projection, view| {
            uniform! {
                equirectangular_map: &source_texture,
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
