use crate::cubemap_loader::CubemapType;
use crate::vertex::Vertex;
use cgmath::Matrix4;
use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::texture::DepthTexture2d;
use glium::Surface;
use glium::Texture2d;
use glium::VertexBuffer;
use glium::{ DrawParameters, Program};
use image::DynamicImage;
use image::ImageBuffer;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

pub struct IrradianceConverter {
    program: Arc<Program>,
    vertex_buffer: Arc<VertexBuffer<Vertex>>,
}

impl IrradianceConverter {
    pub fn load(facade: &impl Facade) -> Self {
        let mut vertex_shader_file =
            File::open("shaders/irradiance_convolution/vertex.glsl").unwrap();
        let mut vertex_shader_src = String::new();
        vertex_shader_file
            .read_to_string(&mut vertex_shader_src)
            .unwrap();
        let mut fragment_shader_file =
            File::open("shaders/irradiance_convolution/fragment.glsl").unwrap();
        let mut fragment_shader_src = String::new();
        fragment_shader_file
            .read_to_string(&mut fragment_shader_src)
            .unwrap();

        let program =
            Program::from_source(facade, &vertex_shader_src, &fragment_shader_src, None).unwrap();

        let vertex_buffer =
            VertexBuffer::new(facade, &crate::material::equirectangle::get_cube_vertices())
                .unwrap();

        Self {
            program: Arc::new(program),
            vertex_buffer: Arc::new(vertex_buffer),
        }
    }

    pub fn calculate_to_fs(
        &self,
        cubemap: CubemapType,
        mut destination_dir: PathBuf,
        extension: &str,
        facade: &impl Facade,
        camera: [[f32; 4]; 4],
    ) {
        // if it is a directory a file needs to be added on to the end of the path
        // the file name and extension will be overwritten later
        if destination_dir.is_dir() {
            destination_dir.push("output.png");
        }

        // Different directions the camera will face to get the output of the shader
        let camera_directions: Vec<Matrix4<f32>> = [
            [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0]],   // right
            [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0]],  // left
            [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],   // top
            [[0.0, -1.0, 0.0], [-1.0, 0.0, 0.0]], // bottom
            [[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],  // front
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],   // back
        ]
        .into_iter()
        .map(|item| Matrix4::look_at_rh([0.0; 3].into(), item[0].into(), item[1].into()))
        .collect();

        // File names directly related to the camera_directions order
        let names: [&str; 6] = [
            "right.jpg",
            "left.jpg",
            "top.jpg",
            "bottom.jpg",
            "front.jpg",
            "back.jpg",
        ];

        let output_size = (32, 32);
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
                environment_map: &cubemap,
                projection: camera,
                view: Into::<[[f32; 4]; 4]>::into(camera_directions[index]),
            };

            frame_buffer.clear_color(1.0, 0.0, 0.0, 0.0);

            frame_buffer
                .draw(
                    &*self.vertex_buffer,
                    index_buffer,
                    &*self.program,
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
