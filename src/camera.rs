use nalgebra::Matrix4;
use nalgebra::Vector3;

const WORLD_UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);

#[derive(Clone)]
pub struct Camera {
    pub forward: Vector3<f32>,
    pub position: Vector3<f32>,
    pub up: Vector3<f32>,
    pub right: Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl Camera {
    pub fn new() -> Self {
        let mut s = Self {
            forward: nalgebra::vector![0.0, 0.0, -1.0],
            position: nalgebra::vector![0.0, 0.0, 0.0],
            up: nalgebra::vector![0.0, 1.0, 0.0],
            right: nalgebra::vector![0.0, 0.0, 0.0],
            yaw: -std::f32::consts::PI / 2.0,
            pitch: 0.0,
            roll: 0.0,
        };

        s.update_vectors();

        s
    }

    fn update_vectors(&mut self) {
        //front.x = cos(glm::radians(Yaw)) * cos(glm::radians(Pitch));
        //front.y = sin(glm::radians(Pitch));
        //front.z = sin(glm::radians(Yaw)) * cos(glm::radians(Pitch));
        self.forward.x = self.yaw.cos() * self.pitch.cos();
        self.forward.y = self.pitch.sin();
        self.forward.z = self.yaw.sin() * self.pitch.cos();

        self.forward = self.forward.normalize();

        self.right = self.forward.cross(&self.up);
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            &self.position.into(),
            &(self.position + self.forward).into(),
            &WORLD_UP,
        )
    }
}
