use cgmath::Rad;
use glium::backend::Facade;
use opengl_render::camera::Camera;
use opengl_render::cubemap_loader::CubemapLoader;
use opengl_render::ibl::Ibl;
use opengl_render::ibl::{IrradianceConverter, Prefilter, BRDF};
use opengl_render::material::{Equirectangle, SkyboxMat, PBR};
use opengl_render::pbr_model::PbrModel;
use opengl_render::skybox::Skybox;
use opengl_render::support::{System, SystemInfo};
use opengl_render::{glium::Surface, renderer::Renderer};
use std::path::PathBuf;

fn main() {
    // Path of the equirectangular texture that will be converted to a cubemap
    let skybox_file = PathBuf::from("./examples/ibl/Summi_Pool/Summi_Pool_3k.hdr");

    // Output directory for generated cubemaps
    let ibl_dir = PathBuf::from("./examples/ibl/Summi_Pool/");

    // Directory of the model to be loaded
    let model_dir = PathBuf::from("./examples/models/primitives/sphere.glb");

    // Create the window and opengl instance
    let mut display = System::init("renderer");
    let facade = display.display.get_context().clone();

    let light_pos = [0.0, 0.4, -10.0];
    let light_color = [300.0, 300.0, 300.0];

    let mut renderer = Renderer::new((*display.display).clone());

    // Convert an equirectangular image into a cubemap and store it to the file system
    // This generated cubemap will be used as the skybox
    let compute = Equirectangle::load_from_fs(&facade);
    compute
        .compute_from_fs_hdr(
            skybox_file,
            ibl_dir.join("cubemap/"),
            "png",
            &facade,
            Camera::new(Rad(std::f32::consts::PI * 0.5), 1024, 1024).into(),
        )
        .unwrap();

    //
    // Here we will generate the irradiance map, prefilter map, and brdf texture
    //
    // The irradiance map maps the light output from the skybox to use as ambient light,
    // The prefilter map is used for reflections
    // The brdf is the same for all skyboxes, we just generate it to make sure that it exists
    //

    // Load the necessary shaders from the file system
    let irradiance_converter = IrradianceConverter::load(&facade);
    let prefilter_shader = Prefilter::load(&facade);
    let brdf_shader = BRDF::new(&facade);

    // Load the skybox again to generate the maps
    let ibl_cubemap =
        CubemapLoader::load_from_fs(ibl_dir.join("cubemap/"), "png", &facade).unwrap();

    // Generate the maps and store them to the file system
    opengl_render::ibl::generate_ibl_from_cubemap(
        &facade,
        &ibl_cubemap,
        ibl_dir.clone(),
        irradiance_converter,
        prefilter_shader,
        brdf_shader,
    )
    .unwrap();

    // Load the skybox from the file system
    let skybox_mat = SkyboxMat::load_from_fs(&facade, ibl_dir.join("cubemap/"), "png").unwrap();
    // Will hold the generated maps
    let mut skybox = Skybox::new(&facade, skybox_mat);

    // Load prefilter, irradiance, and brdf from file system
    let Ibl {
        prefilter,
        irradiance_map: ibl,
        brdf,
    } = opengl_render::ibl::load_ibl_fs(&facade, ibl_dir).unwrap();

    // Assign irradiance map, prefilter map and brdf to the skybox wrapper
    skybox.set_ibl(Some(ibl));
    skybox.set_prefilter(Some(prefilter));
    skybox.set_brdf(Some(brdf));

    // Load the Physically Based Rendering shader from the file system
    let pbr = PBR::load_from_fs(&facade);

    //
    // Here we will load the model that will be rendered
    //

    // This doesn't have to be a vec, but it makes loading multiple models more convenient
    let mut models = vec![PbrModel::load_from_fs(model_dir.clone(), &facade, pbr.clone()).unwrap()];

    models[0].relative_move([0.0, 0.0, 4.0]);

    let camera_pos = [0.0, 0.0, 0.0];

    display.subscribe_render(move |sys_info| {
        let SystemInfo { target, delta, .. } = sys_info;
        let delta_time = delta;
        let frame = target;
        // Time between frames should be used when moving or rotating objects
        let delta_ms = delta_time.as_micros() as f32 / 1000.0;

        // To render a frame, we must begin a new scene.
        // The scene will keep track of variables that apply to the whole scene, like the
        // camera, and skybox.
        let mut scene = renderer.begin_scene();

        // Create a camera with a 60 degree field of view
        let (width, height) = frame.get_dimensions();
        let camera = Camera::new(Rad(std::f32::consts::PI / 3.0), width, height);

        // Set scene variables
        scene.set_camera(camera.get_matrix().into());
        scene.set_camera_pos(camera_pos);
        scene.set_skybox(Some(&skybox));
        scene
            .get_scene_data_mut()
            .get_raw_lights_mut()
            .add_light(light_pos, light_color);

        // send items to be rendered
        // IMPORTANT: you must set the camera position before submitting an object to be
        // rendered. This is because when I add LOD support, it will use the scene's camera
        // position to determine what LOD it should use.
        for model in &models {
            model.render(&mut scene);
        }

        // Render items
        // To render the scene you must give the scene a place to render everything. In order
        // to render to the frame we must first convert it to the Renderable enum, then you can
        // render the scene.
        scene.finish(&mut (*frame).into());

        for model in &mut models {
            model.relative_rotate([Rad(0.0), Rad(0.001 * delta_ms), Rad(0.0)]);
        }
    });

    display.main_loop();
}
