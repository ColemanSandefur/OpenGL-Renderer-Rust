use nalgebra::{Matrix4, Rotation3};

pub struct Rotation {
    rotation: [f32; 3],
}

impl Rotation {
    pub fn from_euler_angles(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self {
            rotation: [roll, pitch, yaw],
        }
    }

    pub fn from_euler_angles_deg(roll: f32, pitch: f32, yaw: f32) -> Self {
        let convert = |value: f32| (value % 360.0).to_radians();
        Self {
            rotation: [convert(roll), convert(pitch), convert(yaw)],
        }
    }

    pub fn get_euler_angles(&self) -> [f32; 3] {
        self.rotation
    }

    pub fn get_euler_angles_deg(&self) -> [f32; 3] {
        [
            self.rotation[0].to_degrees(),
            self.rotation[1].to_degrees(),
            self.rotation[2].to_degrees(),
        ]
    }

    pub fn set_euler_angles(&mut self, roll: f32, pitch: f32, yaw: f32) {
        self.rotation = [roll, pitch, yaw];
    }

    pub fn set_euler_angles_deg(&mut self, roll: f32, pitch: f32, yaw: f32) {
        self.rotation = [roll.to_radians(), pitch.to_radians(), yaw.to_radians()];
    }

    pub fn get_rotation(&self) -> Rotation3<f32> {
        let [roll, pitch, yaw] = self.rotation;
        Rotation3::from_euler_angles(roll, pitch, yaw)
    }

    pub fn from_rotation3(rotation: Rotation3<f32>) -> Self {
        let (roll, pitch, yaw) = rotation.euler_angles();
        Self {
            rotation: [roll, pitch, yaw],
        }
    }

    pub fn get_matrix4(&self) -> Matrix4<f32> {
        let [roll, pitch, yaw] = self.rotation;
        Matrix4::from_euler_angles(roll, pitch, yaw)
    }

    pub fn debug_ui(&mut self, ui: &mut egui::Ui) -> egui::InnerResponse<()> {
        let mut response = None;
        ui.horizontal(|ui| {
            let mut angles = self.get_euler_angles_deg();

            response = Some(
                ui.add(egui::widgets::DragValue::new(&mut angles[0]).prefix("roll: "))
                    | ui.add(egui::widgets::DragValue::new(&mut angles[1]).prefix("pitch: "))
                    | ui.add(egui::widgets::DragValue::new(&mut angles[2]).prefix("yaw: ")),
            );

            self.set_euler_angles_deg(angles[0], angles[1], angles[2]);
        });
        egui::InnerResponse::new((), response.unwrap())
    }
}
