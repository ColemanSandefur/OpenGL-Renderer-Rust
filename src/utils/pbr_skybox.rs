use glium::texture::Cubemap;
use glium::Texture2d;
use std::rc::Rc;

#[derive(Clone)]
pub struct PBRSkybox {
    skybox: Rc<Cubemap>,
    irradiance: Rc<Cubemap>,
    prefilter: Rc<Cubemap>,
    brdf: Rc<Texture2d>,
}

impl PBRSkybox {
    pub fn new(
        skybox: Rc<Cubemap>,
        irradiance: Rc<Cubemap>,
        prefilter: Rc<Cubemap>,
        brdf: Rc<Texture2d>,
    ) -> Self {
        Self {
            skybox,
            irradiance,
            prefilter,
            brdf,
        }
    }
    pub fn set_skybox(&mut self, skybox: Rc<Cubemap>) {
        self.skybox = skybox;
    }

    pub fn get_skybox(&self) -> &Rc<Cubemap> {
        &self.skybox
    }

    pub fn set_irradiance(&mut self, irradiance: Rc<Cubemap>) {
        self.irradiance = irradiance;
    }

    pub fn get_irradiance(&self) -> &Rc<Cubemap> {
        &self.irradiance
    }

    pub fn set_prefilter(&mut self, prefilter: Rc<Cubemap>) {
        self.prefilter = prefilter;
    }

    pub fn get_prefilter(&self) -> &Rc<Cubemap> {
        &self.prefilter
    }
    pub fn set_brdf(&mut self, brdf: Rc<Texture2d>) {
        self.brdf = brdf;
    }

    pub fn get_brdf(&self) -> &Rc<Texture2d> {
        &self.brdf
    }
}
