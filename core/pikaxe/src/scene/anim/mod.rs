mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

#[milo(Anim)]
pub struct AnimObject {}

impl Default for AnimObject {
    fn default() -> AnimObject {
        AnimObject {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // Anim object
            anim_objects: Vec::new(),
            frame: 0.0,
            rate: AnimRate::default(),
        }
    }
}