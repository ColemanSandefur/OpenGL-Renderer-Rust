#[macro_use]
extern crate glium;

use std::error::Error;
use std::path::PathBuf;

use glium::IndexBuffer;
use cgmath::InnerSpace;
use cgmath::Vector3;
use glium::VertexBuffer;
use glium::backend::Facade;
use glium::texture::RawImage2d;
use glium::texture::Texture2d;
use crate::camera::Camera;
use crate::vertex::Vertex;
use image::io::Reader as ImageReader;
use crate::cubemap_loader::{CubemapLoader, CubemapType};
use crate::ibl::{IrradianceConverter, Prefilter, BDRF};
use crate::material::{Equirectangle, PBRParams, SkyboxMat, PBR};
use crate::pbr_model::PbrModel;
use crate::skybox::Skybox;
use crate::support::System;
use crate::{glium::Surface, renderer::Renderer};
use cgmath::Rad;

pub mod basic_model;
pub mod camera;
pub mod cubemap_loader;
pub mod cubemap_render;
pub mod ibl;
pub mod material;
pub mod model;
pub mod pbr_model;
pub mod renderer;
pub mod shape;
pub mod skybox;
pub mod support;
pub mod vertex;

// Rad / ms that should be rotated to get 1 RPM
const RPM: f32 = std::f32::consts::PI * 2.0 / 60.0 / 1000.0;

fn load_texture(facade: &impl Facade, path: PathBuf) -> Result<Texture2d, Box<dyn Error>> {
    let raw_image = ImageReader::open(path)?.decode()?.into_rgb8();

    let source_dimensions = raw_image.dimensions();
    let source_data = raw_image.into_raw();

    let source_image = RawImage2d::from_raw_rgb(source_data, source_dimensions);

    let source_texture = Texture2d::new(facade, source_image)?;

    Ok(source_texture)
}

fn main() {
    let display = System::init("renderer");

    // Light positions should be moved from being stored in the material to stored in the scene
    let light_pos = [0.0, 0.4, -0.7];

    let renderer = Renderer::new((*display.display).clone());

    let compute = Equirectangle::load_from_fs(&*display.display);
    compute.compute_from_fs_hdr(
        "./ibl/Summi_Pool/Summi_Pool_3k.hdr".into(),
        "./ibl/Summi_Pool/cubemap/".into(),
        "png",
        &*display.display,
        Camera::new(Rad(std::f32::consts::PI * 0.5), 1024, 1024).into(),
    );
    let skybox_mat = SkyboxMat::load_from_fs(&*display.display, "./ibl/Summi_Pool/cubemap/", "png");
    let mut skybox = Skybox::new(&*display.display, skybox_mat);

    let irradiance_converter = IrradianceConverter::load(&*display.display);
    let prefilter = Prefilter::load(&*display.display);

    // Calculate irradiance map
    {
        let ibl = CubemapLoader::load_from_fs(
            "./ibl/Summi_Pool/cubemap/".into(),
            "png",
            &*display.display,
        );
        let pf = CubemapLoader::load_from_fs_mipmaps(
            "./ibl/Summi_Pool/cubemap/".into(),
            "png",
            9,
            &*display.display,
        );
        prefilter.calculate_to_fs(
            &pf,
            "./ibl/Summi_Pool/prefilter/".into(),
            "png",
            &*display.display,
            Camera::new(Rad(std::f32::consts::PI * 0.5), 128, 128).into(),
        );
        if let CubemapType::Cubemap(cubemap) = pf {
            println!("There are {} mipmaps", cubemap.get_mipmap_levels());
        }
        irradiance_converter.calculate_to_fs(
            ibl,
            "./ibl/Summi_Pool/ibl_map/".into(),
            "png",
            &*display.display,
            Camera::new(Rad(std::f32::consts::PI * 0.5), 32, 32).into(),
        );
        let bdrf = BDRF::new(&*display.display);
        bdrf.calculate_to_fs(&*display.display, "./ibl/Summi_Pool/brdf.png".into());
    }

    let ibl =
        CubemapLoader::load_from_fs("./ibl/Summi_Pool/ibl_map/".into(), "png", &*display.display);
    skybox.set_ibl(Some(ibl));

    let brdf = load_texture(&*display.display, "./ibl/Summi_Pool/ibl_brdf_lut.png".into()).unwrap();
    skybox.set_brdf(Some(brdf));

    let prefilter = 
        CubemapLoader::load_from_fs("./ibl/Summi_Pool/prefilter/".into(), "png", &*display.display);
    skybox.set_prefilter(Some(prefilter));

    let mut pbr = PBR::load_from_fs(&*display.display);
    pbr.set_light_pos(light_pos);
    pbr.set_pbr_params(PBRParams::metal());

    let mut model = PbrModel::load_from_gltf(
        //"./models/ship/ship.glb".into(),
        "./models/mandalorian/mando.glb".into(),
        &*display.display,
        pbr.clone(),
    );

    for segment in model.get_segments_mut() {
        //segment.get_material_mut().get_pbr_params_mut().metallic = 1.0;
        //segment.get_material_mut().get_pbr_params_mut().roughness = 0.05;
        segment.get_material_mut().get_pbr_params_mut().ao = 1.0;
    }
    let (vertex_buffer, index_buffer) = {
        let width = 1.0;
        let points = [
            [-width / 2.0, width / 2.0, 0.0],
            [width / 2.0, width / 2.0, 0.0],
            [-width / 2.0, -width / 2.0, 0.0],
            [width / 2.0, -width / 2.0, 0.0],
        ];
    let p1: Vector3<f32> = points[0].into();
    let p2: Vector3<f32> = points[1].into();
    let p3: Vector3<f32> = points[2].into();

    let u = p2 - p1;
    let v = p3 - p1;

    let normal: [f32; 3] = u.cross(v).normalize().into();
        let vb = VertexBuffer::new(&*display.display, &[
                Vertex {
                    position: [-width / 2.0, width / 2.0, 0.0],
                    normal,
                    ..Default::default()
                },
                Vertex {
                    position: [width / 2.0, width / 2.0, 0.0],
                    normal,
                    ..Default::default()
                },
                Vertex {
                    position: [-width / 2.0, -width / 2.0, 0.0],
                    normal,
                    ..Default::default()
                },
                Vertex {
                    position: [width / 2.0, -width / 2.0, 0.0],
                    normal,
                    ..Default::default()
                },
            ]).unwrap();
        let ib = IndexBuffer::new(&*display.display, glium::index::PrimitiveType::TrianglesList, &[0u32, 1, 2, 1, 3, 2],).unwrap();

        (vb, ib)
    };

    //let mut model = PbrModel::load_from_mem(vertex_buffer, index_buffer, pbr.clone());

    model.relative_move([0.0, -0.15, 1.0]);
    model.relative_rotate([Rad(0.0), Rad(0.0 * std::f32::consts::PI), Rad(0.0)]);

    let rotation = RPM * 10.0;

    let camera_pos = [0.0, 0.0, 0.0];

    display.main_loop(
        move |_, _| {},
        move |frame, delta_time| {
            let delta_ms = delta_time.as_micros() as f32 / 1000.0;
            //println!(
                //"FPS: {:.0}, Frametime: {:.2}",
                //1.0 / (delta_ms / 1_000.0),
                //delta_ms
            //);

            // Start a new scene
            let mut scene = renderer.begin_scene();

            let (width, height) = frame.get_dimensions();
            let camera = Camera::new(Rad(std::f32::consts::PI / 3.0), width, height);

            // Set scene variables
            scene.set_camera(camera.get_matrix().into());
            scene.set_camera_pos(camera_pos);
            scene.set_skybox(Some(&skybox));

            // send items to be rendered
            model.render(&mut scene);

            // Render items
            scene.finish(&mut frame.into());

            // Manipulate model
            model.relative_rotate([Rad(0.0), Rad(-rotation * delta_ms), Rad(0.0)]);
        },
    );
    println!("Hello, world!");
}

pub fn create_sphere() -> Vec<Vertex> {
    let mut vertices = Vec::new();

    const X_SEGMENTS: u32 = 64;
    const Y_SEGMENTS: u32 = 64;
    const PI: f32 = std::f32::consts::PI;

    for x in 0..X_SEGMENTS {
        for y in 0..Y_SEGMENTS {
            let x_seg = x as f32 / X_SEGMENTS as f32;
            let y_seg = y as f32 / Y_SEGMENTS as f32;

            let x_pos = (x_seg * 2.0 * PI ).cos() * (y_seg * PI).sin();
            let y_pos = (y_seg * PI).cos();
            let z_pos = (x_seg * 2.0 * PI ).sin() * (y_seg * PI).sin();

            vertices.push(Vertex {
                position: [x_pos, y_pos, z_pos],
                tex_coords: [x_seg, y_seg],
                normal: [x_pos, y_pos, z_pos],
                .. Default::default()
            });
        }
    }

    vertices
}
