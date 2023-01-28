use glium::texture::Cubemap;
use std::rc::Rc;

#[derive(Clone)]
pub struct PBRSkybox {
    skybox: Rc<Cubemap>,
    irradiance: Rc<Cubemap>,
}

impl PBRSkybox {
    pub fn new(skybox: Rc<Cubemap>, irradiance: Rc<Cubemap>) -> Self {
        Self { skybox, irradiance }
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
}
