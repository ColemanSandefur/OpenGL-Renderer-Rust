#[macro_use]
extern crate glium;

use crate::material::{PBRParams, SkyboxMat, PBR};
use crate::pbr_model::PbrModel;
use crate::skybox::Skybox;
use crate::support::System;
use crate::{glium::Surface, renderer::Renderer};
use cgmath::Rad;

pub mod basic_model;
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

    let skybox_mat = SkyboxMat::load_from_fs(&*display.display, "./skybox/skybox/front.jpg");
    let skybox = Skybox::new(&*display.display, skybox_mat);

    let mut pbr = PBR::load_from_fs(&*display.display);
    pbr.set_light_pos(light_pos);
    pbr.set_pbr_params(PBRParams::metal());

    let mut model = PbrModel::load_from_fs2(
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
            //println!("Frametime {}", delta_time.as_micros() as f32 / 1000.0);

            // Start a new scene
            let mut scene = renderer.begin_scene();

            let camera = {
                let (width, height) = frame.get_dimensions();
                let aspect_ratio = height as f32 / width as f32;

                let fov = std::f32::consts::PI / 3.0;
                let zfar = 1024.0;
                let znear = 0.10;

                let f = 1.0 / (fov / 2.0).tan();

                [
                    [f * aspect_ratio, 0.0, 0.0, 0.0],
                    [0.0, f, 0.0, 0.0],
                    [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
                    [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0],
                ]
            };

            // Set scene variables
            scene.set_camera(camera.into());
            scene.set_camera_pos(camera_pos);
            scene.set_skybox(Some(&skybox));

            // send items to be rendered
            model.render(&mut scene);

            // Render items
            scene.finish(frame);

            // Manipulate model
            model.relative_rotate([Rad(0.0), Rad(-rotation * delta_ms), Rad(0.0)]);
        },
    );
    println!("Hello, world!");
}
