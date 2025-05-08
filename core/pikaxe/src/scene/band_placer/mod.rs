mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

#[milo]
#[milo_super(Draw, Trans)]
pub struct BandPlacer {
    pub center: String,
}

impl Default for BandPlacer {
    fn default() -> Self {
        // TODO: Match default values to C++ code
        Self {
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

            // BandPlacer object
            center: String::default(),
        }
    }
}