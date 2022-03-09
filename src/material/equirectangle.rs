use crate::camera::Camera;
use crate::cubemap_render::CubemapRender;
use glium::backend::Facade;
use glium::texture::RawImage2d;
use glium::Program;
use glium::Texture2d;
use image::hdr::HdrDecoder;
use image::io::Reader as ImageReader;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
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
        let program = crate::material::insert_program!(
            "../shaders/equirectangle_to_cube/vertex.glsl",
            "../shaders/equirectangle_to_cube/fragment.glsl",
            facade
        );

        Self {
            program: Arc::new(program),
        }
    }

    pub fn compute_from_fs<P>(
        &self,
        source: PathBuf,
        destination_dir: P,
        extension: &str,
        facade: &impl Facade,
        camera: Camera,
    ) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        // let output_size = (1024, 1024);
        let output_size = (2048, 2048);

        let (source_data, source_dimensions) = {
            let mut image = ImageReader::open(source)?.decode()?.into_rgb8();

            let dimensions = image.dimensions();

            for width in 0..dimensions.0 / 2 {
                for height in 0..dimensions.1 {
                    let p = image.get_pixel(width, height).clone();
                    let p2 = image.get_pixel(dimensions.1 - width, height).clone();

                    image.put_pixel(width, height, p2);
                    image.put_pixel(dimensions.1 - width, height, p);
                }
            }

            (image.into_raw(), dimensions)
        };
        let source_image = RawImage2d::from_raw_rgb(source_data, source_dimensions);

        let source_texture = Texture2d::new(facade, source_image)?;

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
        )?;

        Ok(())
    }

    pub fn compute_from_fs_hdr(
        &self,
        source: PathBuf,
        destination_dir: PathBuf,
        extension: &str,
        facade: &impl Facade,
        camera: Camera,
    ) -> Result<(), Box<dyn Error>> {
        let output_size = (1024, 1024);

        let (source_data, source_dimensions) = {
            let buffer = BufReader::new(File::open(&source).ok().ok_or(format!(
                "Unable to load {}",
                source.as_os_str().to_str().unwrap()
            ))?);
            let hdr_image = HdrDecoder::new(buffer)?;
            let dimensions = (hdr_image.metadata().width, hdr_image.metadata().height);

            let mut pixels = hdr_image
                .read_image_hdr()?
                .into_iter()
                .map(|rgb| {
                    return {
                        let values = rgb.0;
                        values
                    };
                })
                .collect::<Vec<[f32; 3]>>();

            for width in 0..dimensions.0 / 2 {
                for height in 0..dimensions.1 {
                    let index = (width + (height * dimensions.0)) as usize;
                    let index2 = ((height * dimensions.0) + (dimensions.0 - width - 1)) as usize;
                    let p = pixels[index];
                    let p2 = pixels[index2];

                    pixels[index] = p2;
                    pixels[index2] = p;
                }
            }

            let output: Vec<f32> = pixels
                .into_iter()
                .flat_map(|rgb| {
                    return {
                        let values = rgb;
                        values
                    };
                })
                .collect();

            (output, dimensions)
        };

        let source_image = RawImage2d::from_raw_rgb(source_data, source_dimensions);

        let source_texture = Texture2d::new(facade, source_image)?;

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
        )?;

        Ok(())
    }
}
