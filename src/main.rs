#[macro_use]
extern crate glium;

use crate::material::{Basic, PBRParams, Phong, Simple, SkyboxMat, PBR};
use crate::pbr_model::PbrModel;
use crate::skybox::Skybox;
use crate::support::System;
use crate::{glium::Surface, renderer::Renderer};
use cgmath::Rad;
use material::Material;

pub mod basic_model;
pub mod material;
pub mod model;
pub mod pbr_model;
pub mod renderer;
pub mod shape;
pub mod skybox;
pub mod support;
pub mod vertex;

const RPM: f32 = std::f32::consts::PI * 2.0 / 60.0 / 1000.0;

fn main() {
    let display = System::init("renderer");

    let renderer = Renderer::new((*display.display).clone());

    let skybox_mat = SkyboxMat::load_from_fs(&display, "./skybox/skybox/front.jpg");
    let skybox = Skybox::new(&*display.display, skybox_mat);

    let simple = Simple::load_from_fs(&*display.display);
    let mut simple2 = simple.clone();
    let mut phong = Phong::load_from_fs(&*display.display);
    phong.set_light_pos([1.4, 0.4, -0.7]);

    let mut pbr = PBR::load_from_fs(&*display.display);
    pbr.set_light_pos([1.4, 0.4, -0.7]);
    pbr.set_pbr_params(PBRParams::metal());

    let mut basic = Basic::load_from_fs(&*display.display);
    basic.set_light_pos([1.4, 0.4, -0.7]);

    println!(
        "{}, {}",
        simple.same_material(&simple2),
        simple.equal(&simple2)
    );

    simple2.set_color([0.0, 1.0, 0.3]);

    println!(
        "{}, {}",
        simple.same_material(&simple2),
        simple.equal(&simple2)
    );

    let mut mando = PbrModel::load_from_fs2("./mando.glb".into(), &*display.display, pbr.clone());
    //let mut mando =
    //PbrModel::load_from_fs("./Mandalorian.obj".into(), &*display.display, pbr.clone());

    mando.relative_move([0.0, -1.0, 3.0]);
    mando.relative_rotate([Rad(0.0), Rad(std::f32::consts::PI), Rad(0.0)]);

    let rotation = RPM * 10.0;

    let mut camera_pos = [0.0, 0.0, 0.0];

    display.main_loop(
        move |_, _| {},
        move |frame, delta_time| {
            let delta_ms = delta_time.as_micros() as f32 / 1000.0;
            //println!("Frametime {}", delta_time.as_micros() as f32 / 1000.0);
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

            scene.set_camera(camera.into());
            scene.set_camera_pos(camera_pos);

            mando.render(&mut scene);
            skybox.render(&mut scene);

            scene.finish(frame);

            mando.relative_rotate([Rad(0.0), Rad(-rotation * delta_ms), Rad(0.0)]);
        },
    );
    println!("Hello, world!");
}
