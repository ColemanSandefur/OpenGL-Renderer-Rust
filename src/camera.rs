use cgmath::{Matrix4, Rad};

/// Basic camera implementation
///
/// A helper struct that represents a camera. fovy is the vertical field of view (in radians),
/// width and height are screen dimensions.
#[derive(Clone, Copy)]
pub struct Camera {
    fovy: Rad<f32>,
    width: u32,
    height: u32,
}

impl Camera {
    /// Farthest point
    pub const ZFAR: f32 = 1024.0;
    /// Nearest point
    pub const ZNEAR: f32 = 0.10;

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

    /// Aspect ratio of the screen (width/height)
    pub fn get_aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn get_fovy(&self) -> &Rad<f32> {
        &self.fovy
    }

    /// The camera matrix to be used in materials.
    ///
    /// Generates the matrix on every call
    pub fn get_matrix(&self) -> Matrix4<f32> {
        let aspect_ratio = self.width as f32 / self.height as f32;

        let f = 1.0 / (self.fovy.0 / 2.0f32).tan();

        let zfar = Self::ZFAR;
        let znear = Self::ZNEAR;

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
