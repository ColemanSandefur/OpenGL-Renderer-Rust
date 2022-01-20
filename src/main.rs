#[macro_use]
extern crate glium;

use crate::material::{PBRParams, Phong, Simple, PBR};
use crate::model::Model;
use crate::pbr_model::PbrModel;
use crate::shape::Shape;
use crate::support::System;
use crate::{glium::Surface, renderer::Renderer};
use cgmath::Rad;
use glium::backend::Facade;
use material::Material;

pub mod material;
pub mod model;
pub mod pbr_model;
pub mod renderer;
pub mod shape;
pub mod support;
pub mod vertex;

const RPM: f32 = std::f32::consts::PI * 2.0 / 60.0 / 1000.0;

fn main() {
    let display = System::init("renderer");

    let renderer = Renderer::new((*display.display).clone());

    let simple = Simple::load_from_fs(&*display.display);
    let mut simple2 = simple.clone();
    let mut phong = Phong::load_from_fs(&*display.display);
    phong.set_light_pos([1.4, 0.4, -0.7]);

    let mut pbr = PBR::load_from_fs(&*display.display);
    pbr.set_light_pos([1.4, 0.4, -0.7]);
    pbr.set_pbr_params(PBRParams::metal());

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

    let mut mando = PbrModel::load_from_fs(
        "./Mandalorian_New.obj".into(),
        &*display.display,
        pbr.clone(),
    );

    //mando.relative_move([0.0, -20.0, 70.0]);
    mando.relative_move([0.0, 0.0, 1.0]);
    mando.relative_rotate([Rad(0.0), Rad(std::f32::consts::PI), Rad(0.0)]);

    //*mando.get_segments_mut()[5]
    //.get_material_mut()
    //.get_pbr_params_mut() = PBRParams::metal();

    //mando.get_segments_mut()[5]
    //.get_material_mut()
    //.get_pbr_params_mut()
    //.metallic = 0.0;

    let rotation = RPM * 20.0;

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
            //camera_pos[2] -= 0.005 * delta_ms;
            scene.set_camera_pos(camera_pos);

            //shape.render(&mut scene);
            //shape2.render(&mut scene);
            mando.render(&mut scene);

            scene.finish(frame);

            //shape.relative_move([0.0, 0.0, 0.005 * delta_ms]);
            //shape.relative_rotate([Rad(0.0), Rad(0.0), Rad(-rotation * delta_ms)]);
            //shape2.relative_move([0.0, 0.0, 0.005 * delta_ms]);
            //mando.relative_move([0.0, 0.0, 0.005 * delta_ms]);
            //mando.relative_rotate([Rad(0.0), Rad(-rotation * delta_ms), Rad(0.0)]);
        },
    );
    println!("Hello, world!");
}
