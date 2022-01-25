use cgmath::{Matrix4, Rad};
pub struct Camera {
    fovy: Rad<f32>,
    width: u32,
    height: u32,
}

impl Camera {
    pub fn new(fovy: Rad<f32>, width: u32, height: u32) -> Self {
        Self {
            fovy,
            width,
            height,
        }
    }

    pub fn set_fovy(&mut self, fovy: Rad<f32>) {
        self.fovy = fovy;
    }
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
    }
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
    }
    pub fn get_matrix(&self) -> Matrix4<f32> {
        let aspect_ratio = self.width as f32 / self.height as f32;

        let f = 1.0 / (self.fovy.0 / 2.0f32).tan();

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
}
