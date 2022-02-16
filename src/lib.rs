//! OpenGL Renderer
//!
//! `opengl_render` is my first real renderer. I have tried to design it to be flexible flexible and still
//! performant.
//!
//! # Basic usage
//!
//! Main work flow is to create a [`Renderer`] and load the necessary [`Materials`]. Every frame the [`Renderer`] will create a
//! [`RenderScene`]. This scene will hold scene specific data. You can add a model to be rendered
//! by either passing the index buffer, vertex buffer, and material to the scene. Or You can use a
//! [`Model`] which will manage your index and vertex buffers, and its material. You can call
//! [`render`] on a model and it will submit itself to the renderer. However you submit to the
//! [`RenderScene`], it will hold a reference to the buffers and material. Therefore, you must keep
//! the [`Model`] or buffers alive longer than the [`RenderScene`]. When you are done adding things
//! to render, you call [`finish`](crate::renderer::RenderScene::finish) on the scene and it will
//! consume itself and render all it's contents. After this, you are safe to drop the model or
//! buffer if needed.
//!
//! [`Renderer`]: crate::renderer::Renderer
//! [`Materials`]: crate::material::Material
//! [`RenderScene`]: crate::renderer::RenderScene
//! [`Model`]: crate::model::Model
//! [`render`]: crate::model::Model::render
//!
//! # Example
//!
//! ```
//! use std::path::PathBuf;
//! use opengl_render::ibl::Ibl;
//! use cgmath::Rad;
//! use opengl_render::camera::Camera;
//! use opengl_render::cubemap_loader::{CubemapLoader};
//! use opengl_render::ibl::{IrradianceConverter, Prefilter, BRDF};
//! use opengl_render::material::{Equirectangle, SkyboxMat, PBR};
//! use opengl_render::pbr_model::PbrModel;
//! use opengl_render::skybox::Skybox;
//! use opengl_render::support::System;
//! use opengl_render::{glium::Surface, renderer::Renderer};
//!
//! fn main() {
//!     // Path of the equirectangular texture that will be converted to a cubemap
//!     let skybox_file = PathBuf::from("./examples/ibl/Summi_Pool/Summi_Pool_3k.hdr");
//!
//!     // Output directory for generated cubemaps
//!     let ibl_dir = PathBuf::from("./examples/ibl/Summi_Pool/");
//!
//!     // Directory of the model to be loaded
//!     let model_dir = PathBuf::from("./examples/models/primitives/cube.obj");
//!
//!     // Create the window and opengl instance
//!     let display = System::init("renderer");
//!
//!     // Light positions should be moved from being stored in the material to stored in the scene
//!     let light_pos = [0.0, 0.4, -10.0];
//!
//!     let renderer = Renderer::new((*display.display).clone());
//!
//!     // Convert an equirectangular image into a cubemap and store it to the file system
//!     // This generated cubemap will be used as the skybox
//!     let compute = Equirectangle::load_from_fs(&*display.display);
//!     compute.compute_from_fs_hdr(
//!         skybox_file,
//!         ibl_dir.join("cubemap/"),
//!         "png",
//!         &*display.display,
//!         Camera::new(Rad(std::f32::consts::PI * 0.5), 1024, 1024).into(),
//!     );
//!
//!     //
//!     // Here we will generate the irradiance map, prefilter map, and brdf texture
//!     //
//!     // The irradiance map maps the light output from the skybox to use as ambient light,
//!     // The prefilter map is used for reflections
//!     // The brdf is the same for all skyboxes, we just generate it to make sure that it exists
//!     //
//!
//!     // Load the necessary shaders from the file system
//!     let irradiance_converter = IrradianceConverter::load(&*display.display);
//!     let prefilter_shader = Prefilter::load(&*display.display);
//!     let brdf_shader = BRDF::new(&*display.display);
//!
//!     // Load the skybox again to generate the maps
//!     let ibl_cubemap = CubemapLoader::load_from_fs(
//!         ibl_dir.join("cubemap/"),
//!         "png",
//!         &*display.display,
//!     );
//!
//!     // Generate the maps and store them to the file system
//!     opengl_render::ibl::generate_ibl_from_cubemap(&*display.display, &ibl_cubemap, ibl_dir.clone(), irradiance_converter, prefilter_shader, brdf_shader);
//!
//!     // Load the skybox from the file system
//!     let skybox_mat = SkyboxMat::load_from_fs(&*display.display, ibl_dir.join("cubemap/"), "png");
//!     // Will hold the generated maps
//!     let mut skybox = Skybox::new(&*display.display, skybox_mat);
//!
//!     // Load prefilter, irradiance, and brdf from file system
//!     let Ibl {
//!         prefilter,
//!         irradiance_map: ibl,
//!         brdf,
//!     } = opengl_render::ibl::load_ibl_fs(&*display.display, ibl_dir);
//!
//!     // Assign irradiance map, prefilter map and brdf to the skybox wrapper
//!     skybox.set_ibl(Some(ibl));
//!     skybox.set_prefilter(Some(prefilter));
//!     skybox.set_brdf(Some(brdf));
//!
//!     // Load the Physically Based Rendering shader from the file system
//!     let mut pbr = PBR::load_from_fs(&*display.display);
//!     pbr.set_light_pos(light_pos);
//!
//!     //
//!     // Here we will load the model that will be rendered
//!     //
//!
//!     // This doesn't have to be a vec, but it makes loading multiple models more convenient
//!     let mut models = vec![PbrModel::load_from_fs(
//!         model_dir,
//!         &*display.display,
//!         pbr.clone(),
//!     )];
//!
//!     models[0].relative_move([0.0, 0.0, 4.0]);
//!
//!     let camera_pos = [0.0, 0.0, 0.0];
//!
//!     display.main_loop(
//!         move |_, _| {},
//!         move |frame, delta_time| {
//!             // Time between frames should be used when moving or rotating objects
//!             let delta_ms = delta_time.as_micros() as f32 / 1000.0;
//!
//!             // To render a frame, we must begin a new scene.
//!             // The scene will keep track of variables that apply to the whole scene, like the
//!             // camera, and skybox.
//!             let mut scene = renderer.begin_scene();
//!
//!             // Create a camera with a 60 degree field of view
//!             let (width, height) = frame.get_dimensions();
//!             let camera = Camera::new(Rad(std::f32::consts::PI / 3.0), width, height);
//!
//!             // Set scene variables
//!             scene.set_camera(camera.get_matrix().into());
//!             scene.set_camera_pos(camera_pos);
//!             scene.set_skybox(Some(&skybox));
//!
//!             // send items to be rendered
//!             // IMPORTANT: you must set the camera position before submitting an object to be
//!             // rendered. This is because when I add LOD support, it will use the scene's camera
//!             // position to determine what LOD it should use.
//!             for model in &models {
//!                 model.render(&mut scene);
//!             }
//!
//!             // Render items
//!             // To render the scene you must give the scene a place to render everything. In order
//!             // to render to the frame we must first convert it to the Renderable enum, then you can
//!             // render the scene.
//!             scene.finish(&mut frame.into());
//!         },
//!     );
//! }
//!
//! ```

#[macro_use]
pub extern crate glium;

pub extern crate cgmath;
pub extern crate egui;
pub extern crate image;

pub mod camera;
pub mod cubemap_loader;
pub mod cubemap_render;
pub mod gui;
pub mod ibl;
pub mod material;
pub mod model;
pub mod pbr_model;
pub mod renderer;
pub mod skybox;
pub mod support;
pub mod texture;
pub mod vertex;
