use cgmath::Matrix4;
use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::DepthTexture2d;
use glium::texture::RawImage2d;
use glium::Surface;
use glium::Texture2d;
use glium::VertexBuffer;
use glium::{ DrawParameters, Program};
use image::hdr::HdrDecoder;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use image::ImageBuffer;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use crate::vertex::Vertex;

#[derive(Clone)]
pub struct Equirectangle {
    program: Arc<Program>,
    vertex_buffer: Arc<VertexBuffer<Vertex>>,
}

impl Equirectangle {
    pub fn load_from_fs(facade: &impl Facade) -> Self {
        let mut vertex_shader_file =
            File::open("shaders/equirectangle_to_cube/vertex.glsl").unwrap();
        let mut vertex_shader_src = String::new();
        vertex_shader_file
            .read_to_string(&mut vertex_shader_src)
            .unwrap();
        let mut fragment_shader_file =
            File::open("shaders/equirectangle_to_cube/fragment.glsl").unwrap();
        let mut fragment_shader_src = String::new();
        fragment_shader_file
            .read_to_string(&mut fragment_shader_src)
            .unwrap();

        let program =
            Program::from_source(facade, &vertex_shader_src, &fragment_shader_src, None).unwrap();

        let vertex_buffer = VertexBuffer::new(facade, &get_cube_vertices()).unwrap();

        Self {
            program: Arc::new(program),
            vertex_buffer: Arc::new(vertex_buffer),
        }
    }

    pub fn compute_from_fs(
        &self,
        source: PathBuf,
        mut destination_dir: PathBuf,
        extension: &str,
        facade: &impl Facade,
        camera: [[f32; 4]; 4],
    ) {
        if destination_dir.is_dir() {
            destination_dir.push("random.png");
        }
        let camera_directions: Vec<Matrix4<f32>> = [
            [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],
            [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0]],
            [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
            [[0.0, -1.0, 0.0], [-1.0, 0.0, 0.0]],
            [[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        ]
        .into_iter()
        .map(|item| Matrix4::look_at_rh([0.0; 3].into(), item[0].into(), item[1].into()))
        .collect();

        let names: [&str; 6] = ["right", "left", "top", "bottom", "front", "back"];

        let (source_data, source_dimensions) = {
            let image = ImageReader::open(source)
                .unwrap()
                .decode()
                .unwrap()
                .into_rgb8();

            let dimensions = image.dimensions();

            (image.into_raw(), dimensions)
        };

        let output_size = (1024, 1024);

        let source_image = RawImage2d::from_raw_rgb(source_data, source_dimensions);

        let source_texture = Texture2d::new(facade, source_image).unwrap();

        let output_texture = Texture2d::empty_with_format(
            facade,
            glium::texture::UncompressedFloatFormat::U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            output_size.0,
            output_size.1,
        )
        .unwrap();
        let depth = DepthTexture2d::empty(facade, output_size.0, output_size.1).unwrap();

        let mut frame_buffer =
            SimpleFrameBuffer::with_depth_buffer(facade, &output_texture, &depth).unwrap();

        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        for index in 0..names.len() {
            let uniforms = uniform! {
                equirectangular_map: &source_texture,
                projection: camera,
                view: Into::<[[f32; 4]; 4]>::into(camera_directions[index]),
            };

            frame_buffer
                .draw(
                    &*self.vertex_buffer,
                    index_buffer,
                    &*self.program,
                    &uniforms,
                    &DrawParameters {
                        ..Default::default()
                    },
                )
                .unwrap();

            let mut output = Vec::new();
            for pixel in output_texture.read_to_pixel_buffer().read().unwrap() {
                output.push(pixel.0);
                output.push(pixel.1);
                output.push(pixel.2);
                output.push(pixel.3);
            }

            let output_image = DynamicImage::ImageRgba8(
                ImageBuffer::from_raw(output_size.0, output_size.1, output).unwrap(),
            );

            output_image
                .save(
                    destination_dir
                        .with_file_name(names[index])
                        .with_extension(extension),
                )
                .unwrap();
        }
    }
    pub fn compute_from_fs_hdr(
        &self,
        source: PathBuf,
        mut destination_dir: PathBuf,
        extension: &str,
        facade: &impl Facade,
        camera: [[f32; 4]; 4],
    ) {
        if destination_dir.is_dir() {
            destination_dir.push("random.png");
        }
        let camera_directions: Vec<Matrix4<f32>> = [
            [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],
            [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0]],
            [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
            [[0.0, -1.0, 0.0], [-1.0, 0.0, 0.0]],
            [[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        ]
        .into_iter()
        .map(|item| Matrix4::look_at_rh([0.0; 3].into(), item[0].into(), item[1].into()))
        .collect();

        let names: [&str; 6] = [
            "right.jpg",
            "left.jpg",
            "top.jpg",
            "bottom.jpg",
            "front.jpg",
            "back.jpg",
        ];

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

        let output_size = (1024, 1024);

        let source_image = RawImage2d::from_raw_rgb(source_data, source_dimensions);

        let source_texture = Texture2d::new(facade, source_image).unwrap();

        let output_texture = Texture2d::empty_with_format(
            facade,
            glium::texture::UncompressedFloatFormat::F16F16F16,
            glium::texture::MipmapsOption::NoMipmap,
            output_size.0,
            output_size.1,
        )
        .unwrap();
        let depth = DepthTexture2d::empty(facade, output_size.0, output_size.1).unwrap();

        let mut frame_buffer =
            SimpleFrameBuffer::with_depth_buffer(facade, &output_texture, &depth).unwrap();

        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        for index in 0..names.len() {
            let uniforms = uniform! {
                equirectangular_map: &source_texture,
                projection: camera,
                view: Into::<[[f32; 4]; 4]>::into(camera_directions[index]),
            };

            frame_buffer
                .draw(
                    &*self.vertex_buffer,
                    index_buffer,
                    &*self.program,
                    &uniforms,
                    &DrawParameters {
                        ..Default::default()
                    },
                )
                .unwrap();

            let mut bytes = Vec::new();
            unsafe {
                let pb = output_texture.unchecked_read_to_pixel_buffer::<(f32, f32, f32)>();

                for pixel in pb.read().unwrap() {
                    bytes.push(pixel.0);
                    bytes.push(pixel.1);
                    bytes.push(pixel.2);
                }
            }

            let mut output = Vec::new();
            for pixel in output_texture.read_to_pixel_buffer().read().unwrap() {
                output.push(pixel.0);
                output.push(pixel.1);
                output.push(pixel.2);
                output.push(pixel.3);
            }

            let output_image = DynamicImage::ImageRgba8(
                ImageBuffer::from_raw(output_size.0, output_size.1, output).unwrap(),
            );

            output_image
                .save(
                    destination_dir
                        .with_file_name(names[index])
                        .with_extension(extension),
                )
                .unwrap();
        }
    }
}

pub fn get_cube_vertices() -> Vec<Vertex> {
    let output = vec![
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, -1.0, -1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [1.0, -1.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-left
        Vertex {
            position: [-1.0, -1.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [-1.0, -1.0, -1.0],
            normal: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [1.0, 1.0, -1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 1.0],
            ..Default::default()
        }, // top-right
        Vertex {
            position: [1.0, 1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            ..Default::default()
        }, // bottom-right
        Vertex {
            position: [-1.0, 1.0, -1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            ..Default::default()
        }, // top-left
        Vertex {
            position: [-1.0, 1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            ..Default::default()
        }, // bottom-left
    ];

    output
}
