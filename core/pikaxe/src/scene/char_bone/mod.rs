mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

#[milo(Trans)]
pub struct CharBone {
    pub position: bool,
    pub scale: bool,
    pub rotation: RotationConstraint,
    pub unknown: f32,
}

impl Default for CharBone {
    fn default() -> CharBone {
        CharBone {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // Trans object
            local_xfm: Matrix::default(),
            world_xfm: Matrix::default(),

            trans_objects: Vec::new(),

            constraint: TransConstraint::default(),
            target: String::default(),

            preserve_scale: false,
            parent: String::default(),

            // CharBone object
            position: true,
            scale: true,
            rotation: RotationConstraint::kRotFull,
            unknown: 1.0,
        }
    }
}