mod io;

use super::{Quat, Vector3};
use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

// TODO: Combine with keys used by prop anim
//  Maybe rename to AnimKey...
#[derive(Debug, Default)]
pub struct AnimEvent<T: std::fmt::Debug + Default> {
    pub value: T,
    pub pos: f32
}

// TODO: Probably add TransAnim to derive macro
#[milo(Anim)]
pub struct TransAnim {
    pub trans_object: String,
    pub rot_keys: Vec<AnimEvent<Quat>>,
    pub trans_keys: Vec<AnimEvent<Vector3>>,
    pub trans_anim_owner: String,
    pub trans_spline: bool,
    pub repeat_trans: bool,
    pub scale_keys: Vec<AnimEvent<Vector3>>,
    pub scale_spline: bool,
    pub follow_path: bool,
    pub rot_slerp: bool,
    pub rot_spline: bool
}

impl Default for TransAnim {
    fn default() -> TransAnim {
        TransAnim {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // Anim object
            anim_objects: Vec::new(),
            frame: 0.0,
            rate: AnimRate::default(),

            // TransAnim object
            trans_object: String::default(),
            rot_keys: Vec::new(),
            trans_keys: Vec::new(),
            trans_anim_owner: String::default(),
            trans_spline: false,
            repeat_trans: false,
            scale_keys: Vec::new(),
            scale_spline: false,
            follow_path: false,
            rot_slerp: false,
            rot_spline: false
        }
    }
}