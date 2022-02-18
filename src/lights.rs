/// A simple struct to hold lights
///
/// It holds each component of the light in a separate vector (color is its own vec, position is
/// its own vec, etc.).
pub struct RawLights {
    colors: Vec<[f32; 3]>,
    positions: Vec<[f32; 3]>,
}

impl RawLights {
    /// Returns position and color of light
    pub fn get_light(&self, index: usize) -> (&[f32; 3], &[f32; 3]) {
        (&self.positions[index], &self.colors[index])
    }

    /// Returns a tuple containing all light positions and light colors
    pub fn get_lights(&self) -> (&Vec<[f32; 3]>, &Vec<[f32; 3]>) {
        (&self.positions, &self.colors)
    }

    pub fn add_light(&mut self, position: [f32; 3], color: [f32; 3]) {
        self.colors.push(color);
        self.positions.push(position);
    }

    pub fn new() -> Self {
        Self {
            colors: Vec::new(),
            positions: Vec::new(),
        }
    }
}
