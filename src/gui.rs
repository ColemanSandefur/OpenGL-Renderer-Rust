use std::ops::RangeInclusive;

use egui::Response;

/// Print object debug info to gui.
///
/// Using egui, you can add debug information to the passed in ui. This can allow you to easily
/// edit values during run-time (like object color). In general, the implementation should just
/// care about printing its own members, not creating a whole new section for itself.
pub trait DebugGUI {
    fn debug(&mut self, ui: &mut egui::Ui);
}

/// A list of helpful egui formatting functions
///
/// There are a few things (like position sliders) that would be duplicated around the codebase, so
/// instead I am putting the common functions here (position sliders is called the `position`
/// function). One of the most helpful one is `multi_slider` where it will create a slider for every value passed
/// in.
pub struct DebugGUIFormat;

impl DebugGUIFormat {
    /// Add multiple sliders to the ui.
    ///
    /// You can supply a buffer of values where each value will become a slider. You can also
    /// supply prefixes for each slider. If there are less names provided than values provided,
    /// names will be treated as a None value.
    pub fn multi_slider<T: egui::emath::Numeric>(ui: &mut egui::Ui, values: &mut [T], names: Option<&[&str]>, range: RangeInclusive<T>) -> bool {
        let mut changed = false;

        if names.is_none() || names.as_ref().unwrap().len() < values.len() {
            for i in 0..values.len() {
                changed = changed | ui.add(egui::Slider::new(&mut values[i], range.clone()).max_decimals(2)).changed();
            }
        } else {
            let names = names.unwrap();
            for i in 0..values.len() {
                changed = changed | ui.add(egui::Slider::new(&mut values[i], range.clone()).max_decimals(2).prefix(format!("{}:", names[i]))).changed();
            }
        }

        changed
    }

    /// rgb color picker
    ///
    /// Simple wrapper of egui color_picker, but basically just an alias
    pub fn rgb(ui: &mut egui::Ui, rgb: &mut [f32; 3]) -> Response {
        egui::widgets::color_picker::color_edit_button_rgb(ui, rgb)
    }

    /// rgb color picker
    ///
    /// Convenient wrapper around egui's color_picker which accepts f32s but glium's textures are
    /// often read in bytes so this converts between them automatically.
    pub fn rgb_byte(ui: &mut egui::Ui, rgb: &mut [u8; 3]) -> Response {
        // Convert from 0..256 to 0.0..1.0
        let mut f = [
            rgb[0] as f32 / 255.0,
            rgb[1] as f32 / 255.0,
            rgb[2] as f32 / 255.0,
        ];

        let result = egui::widgets::color_picker::color_edit_button_rgb(ui, &mut f);

        // Convert from 0.0..1.0 to 0..256
        *rgb = [
            (f[0] * 255.0) as u8,
            (f[1] * 255.0) as u8,
            (f[2] * 255.0) as u8,
        ];

        result
    }

    /// Simple rotation sliders
    ///
    /// A wrapper for multi_slider where the labels for axis are x, y, z. It also formats the
    /// rotation to be between 0 and 2 PI.
    pub fn rotation(ui: &mut egui::Ui, rad: &mut [f32; 3]) -> bool {
        rad[0] %= std::f32::consts::PI * 2.0;
        rad[1] %= std::f32::consts::PI * 2.0;
        rad[2] %= std::f32::consts::PI * 2.0;

        DebugGUIFormat::multi_slider(ui, rad, Some(&["x", "y", "z"]), 0.0..=std::f32::consts::PI * 2.0)
    }

    /// Simple position sliders
    ///
    /// A wrapper for multi_slider where the labels for axis are x, y, z.
    pub fn position(ui: &mut egui::Ui, pos: &mut [f32; 3], range: RangeInclusive<f32>) -> bool {
        DebugGUIFormat::multi_slider(ui, pos, Some(&["x", "y", "z"]), range)
    }
}
