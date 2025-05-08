mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

#[milo]
#[milo_super(Draw, Trans)]
pub struct CamObject {
    pub near_plane: f32,
    pub far_plane: f32,
    pub y_fov: f32,

    pub screen_rect: Rect,
    pub z_range: Vector2,
    pub target_tex: String,
}

impl Default for CamObject {
    fn default() -> CamObject {
        // TODO: Match default values to C++ code
        CamObject {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // Draw object
            showing: true,
            draw_objects: Vec::new(),
            sphere: Sphere::default(),
            draw_order: 0.0,
            override_include_in_depth_only_pass: OverrideIncludeInDepthOnlyPass::default(),

            // Trans object
            local_xfm: Matrix::default(),
            world_xfm: Matrix::default(),

            trans_objects: Vec::new(),

            constraint: TransConstraint::default(),
            target: String::default(),

            preserve_scale: false,
            parent: String::default(),

            // Cam object
            near_plane: 0.0,
            far_plane: 0.0,
            y_fov: 0.0,

            screen_rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0
            },
            z_range: Vector2::default(),
            target_tex: String::default(),
        }
    }
}