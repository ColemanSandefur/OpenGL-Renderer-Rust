#[macro_use]
extern crate glium;

use crate::cubemap_loader::CubemapLoader;
use crate::ibl::IrradianceConverter;
use crate::material::{Equirectangle, PBRParams, SkyboxMat, PBR};
use crate::pbr_model::PbrModel;
use crate::skybox::Skybox;
use crate::support::System;
use crate::{glium::Surface, renderer::Renderer};
use cgmath::{Matrix4, Rad};

pub mod basic_model;
pub mod cubemap_loader;
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
        create_camera(Rad(std::f32::consts::PI * 0.5), 1024.0, 1024.0).into(),
    );
    let skybox_mat = SkyboxMat::load_from_fs(&*display.display, "./ibl/Summi_Pool/cubemap/", "png");
    //let skybox_mat = SkyboxMat::load_from_memory(&*display.display, images, 1024, 1024);
    let mut skybox = Skybox::new(&*display.display, skybox_mat);

    let irradiance_converter = IrradianceConverter::load(&*display.display);

    // Calculate irradiance map
    {
        let ibl = CubemapLoader::load_from_fs(
            "./ibl/Summi_Pool/cubemap/".into(),
            "png",
            &*display.display,
        );
        irradiance_converter.calculate_to_fs(
            ibl,
            "./ibl/Summi_Pool/ibl_map/".into(),
            "png",
            &*display.display,
            create_camera(Rad(std::f32::consts::PI * 0.5), 32.0, 32.0).into(),
        );
    }

    let ibl =
        CubemapLoader::load_from_fs("./ibl/Summi_Pool/ibl_map/".into(), "png", &*display.display);

    skybox.set_ibl(Some(ibl));

    let mut pbr = PBR::load_from_fs(&*display.display);
    pbr.set_light_pos(light_pos);
    pbr.set_pbr_params(PBRParams::metal());

    let mut model = PbrModel::load_from_gltf(
        //"./models/ship/ship.glb".into(),
        "./models/mandalorian/mando.glb".into(),
        &*display.display,
        pbr.clone(),
    );

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
            let camera: Matrix4<f32> =
                create_camera(Rad(std::f32::consts::PI / 3.0), width as f32, height as f32);

            // Set scene variables
            scene.set_camera(camera.into());
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

fn create_camera(fovy: Rad<f32>, width: f32, height: f32) -> Matrix4<f32> {
    let aspect_ratio = width as f32 / height as f32;

    let f = 1.0 / (fovy.0 / 2.0f32).tan();

    let zfar = 1024.0;
    let znear = 0.10;

    [
        [f / aspect_ratio, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [
            0.0,
            0.0,
            (zfar + znear) / (zfar - znear),
            (2.0 * (zfar + znear)) / (2.0 * (zfar - znear)),
        ],
        [0.0, 0.0, -1.0, 0.0],
    ]
    .into()
}
