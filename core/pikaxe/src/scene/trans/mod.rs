mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

#[milo(Trans)]
pub struct TransObject {}

impl Default for TransObject {
    fn default() -> TransObject {
        TransObject {
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
        }
    }
}