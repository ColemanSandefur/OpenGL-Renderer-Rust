#[macro_use]
extern crate glium;

use crate::glium::GlObject;
use std::error::Error;
use std::path::PathBuf;

use crate::camera::Camera;
use crate::cubemap_loader::{CubemapLoader, CubemapType};
use crate::ibl::{IrradianceConverter, Prefilter, BDRF};
use crate::material::{Equirectangle, PBRParams, SkyboxMat, PBR};
use crate::pbr_model::PbrModel;
use crate::skybox::Skybox;
use crate::support::System;
use crate::{glium::Surface, renderer::Renderer};
use cgmath::Rad;
use cgmath::Vector3;
use glium::backend::Facade;
use glium::texture::RawImage2d;
use glium::texture::Texture2d;
use image::io::Reader as ImageReader;
use material::PBRTextures;

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

    unsafe {
        let texture = source_texture.get_id();

        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    }

    Ok(source_texture)
}

fn main() {
    let display = System::init("renderer");

    // Light positions should be moved from being stored in the material to stored in the scene
    let light_pos = [0.0, 0.4, -10.0];

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
        let pf = CubemapLoader::load_from_fs(
            "./ibl/Summi_Pool/cubemap/".into(),
            "png",
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

    let brdf = load_texture(
        &*display.display,
        "./ibl/Summi_Pool/ibl_brdf_lut.png".into(),
    )
    .unwrap();
    skybox.set_brdf(Some(brdf));

    let prefilter = CubemapLoader::load_mips_fs(
        "./ibl/Summi_Pool/prefilter/".into(),
        "png",
        &*display.display,
    );

    match &prefilter {
        CubemapType::Cubemap(c) => println!("mips: {}", c.get_mipmap_levels()),
        CubemapType::SrgbCubemap(c) => println!("mips: {}", c.get_mipmap_levels()),
    };
    skybox.set_prefilter(Some(prefilter));

    let mut pbr = PBR::load_from_fs(&*display.display);
    pbr.set_light_pos(light_pos);

    // Load model

    //pbr.get_pbr_params_mut().roughness = 0.05;

    //let mut models = generate_cubes(
    //7,
    //7,
    //[0.0, 0.0, 30.0].into(),
    //&*display.display,
    //pbr.clone(),
    //);

    let mut models = vec![PbrModel::load_from_gltf(
        "./models/mandalorian/mando.glb".into(),
        &*display.display,
        pbr.clone(),
    )];
    models[0].relative_move([0.0, -1.0, 10.0]);

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
            for model in &models {
                model.render(&mut scene);
            }

            // Render items
            scene.finish(&mut frame.into());

            // Manipulate model
            for model in &mut models {
                model.relative_rotate([Rad(0.0), Rad(-rotation * delta_ms), Rad(0.0)]);
            }
        },
    );
    println!("Hello, world!");
}

fn generate_cubes(
    width: u32,
    height: u32,
    offset: Vector3<f32>,
    facade: &impl Facade,
    pbr: PBR,
) -> Vec<PbrModel> {
    let mut models = Vec::new();

    let gap = 3.0;
    let top_left = offset
        - Vector3::new(
            (width - 1) as f32 / 2.0 * gap,
            (height - 1) as f32 / 2.0 * gap,
            0 as f32,
        );

    for row in 0..height {
        let metallic = row as f32 / height as f32;
        for column in 0..width {
            let roughness = (column as f32 / width as f32).max(0.05);
            let mut model = PbrModel::load_from_gltf(
                //"./models/ship/ship.glb".into(),
                "./models/cube/sphere.glb".into(),
                &*facade,
                pbr.clone(),
            );

            {
                let segments = model.get_segments_mut();
                let mut material = PBRParams::default();

                material.albedo = [0.5, 0.0, 0.0].into();
                material.metallic = metallic;
                material.roughness = roughness;

                segments[0]
                    .get_material_mut()
                    .set_pbr_params(PBRTextures::from_params(material, facade));
            }

            model
                .relative_move(top_left + Vector3::new(gap * column as f32, gap * row as f32, 0.0));

            models.push(model);
        }
    }

    models
}
