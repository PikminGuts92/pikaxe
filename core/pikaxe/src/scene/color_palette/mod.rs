mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

pub struct PaletteColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub unknown: f32, // TODO: figure out what this is
}

#[milo]
pub struct ColorPalette {
    pub num_colors: u32,
    pub colors: Vec<PaletteColor>,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // ColorPalette object
            num_colors: 0,
            colors: Vec::new(),
        }
    }
}