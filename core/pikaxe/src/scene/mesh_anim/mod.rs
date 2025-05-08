mod io;

use super::{AnimEvent, Color4, Vector2, Vector3};
use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

// TODO: Probably add MeshAnim to derive macro
#[milo(Anim)]
pub struct MeshAnim {
    pub mesh: String,
    pub vert_point_keys: Vec<AnimEvent<Vec<Vector3>>>,
    pub vert_text_keys: Vec<AnimEvent<Vec<Vector2>>>,
    pub vert_color_keys: Vec<AnimEvent<Vec<Color4>>>,
    pub keys_owner: String,
}

impl Default for MeshAnim {
    fn default() -> MeshAnim {
        MeshAnim {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // Anim object
            anim_objects: Vec::new(),
            frame: 0.0,
            rate: AnimRate::default(),

            // MeshAnim object
            mesh: String::default(),
            vert_point_keys: Vec::new(),
            vert_text_keys: Vec::new(),
            vert_color_keys: Vec::new(),
            keys_owner: String::default(),
        }
    }
}