pub mod irradiance_conversion;
pub mod specular;

pub use irradiance_conversion::*;
pub use specular::*;

use crate::{camera::Camera, texture::TextureLoader};
use cgmath::Rad;
use std::path::PathBuf;
use glium::{backend::Facade, Texture2d};

use crate::cubemap_loader::{CubemapType, CubemapLoader};

pub struct Ibl {
    pub irradiance_map: CubemapType,
    pub prefilter: CubemapType,
    pub brdf: Texture2d,
}

// given a cubemap, this function will generate all the necessary files to be used for image based
// lighting
pub fn generate_ibl_from_cubemap(facade: &impl Facade, cubemap: &CubemapType, output_directory: PathBuf, ir: IrradianceConverter, prefilter: Prefilter, brdf: BDRF){
    let pf_dir = output_directory.join("prefilter");
    let ir_dir = output_directory.join("ibl_map");
    let brdf_dir = output_directory.join("brdf.png");

    prefilter.calculate_to_fs(
        cubemap,
        pf_dir,
        "png",
        facade,
        Camera::new(Rad(std::f32::consts::PI * 0.5), 128, 128).into(),
    );
    ir.calculate_to_fs(
        cubemap,
        ir_dir,
        "png",
        facade,
        Camera::new(Rad(std::f32::consts::PI * 0.5), 32, 32).into(),
    );
    brdf.calculate_to_fs(facade, brdf_dir);
}

pub fn load_ibl_fs(facade: &impl Facade, directory: PathBuf) -> Ibl {
    let pf_dir = directory.join("prefilter");
    let ir_dir = directory.join("ibl_map");
    let brdf_dir = directory.join("brdf.png");

    let ir_map =
        CubemapLoader::load_from_fs(ir_dir, "png", facade);
    let pf_map = 
        CubemapLoader::load_mips_fs(pf_dir, "png", facade);
    let brdf = TextureLoader::from_fs(facade, &brdf_dir).unwrap();

    Ibl {
        irradiance_map: ir_map,
        prefilter: pf_map,
        brdf 
    }
}
