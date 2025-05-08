mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

#[milo(Group)]
#[milo_super(Anim, Draw, Trans)]
pub struct GroupObject {}

impl Default for GroupObject {
    fn default() -> GroupObject {
        GroupObject {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // Anim object
            anim_objects: Vec::new(),
            frame: 0.0,
            rate: AnimRate::default(),

            // Trans object
            local_xfm: Matrix::default(),
            world_xfm: Matrix::default(),

            trans_objects: Vec::new(),

            constraint: TransConstraint::default(),
            target: String::default(),

            preserve_scale: false,
            parent: String::default(),

            // Draw object
            showing: true,
            draw_objects: Vec::new(),
            sphere: Sphere::default(),
            draw_order: 0.0,
            override_include_in_depth_only_pass: OverrideIncludeInDepthOnlyPass::default(),

            // Group object
            objects: Vec::new(),
            environ: String::default(),
            draw_only: String::default(),
            lod: String::default(),
            lod_screen_size: 0.0,
            sort_in_world: false,
        }
    }
}