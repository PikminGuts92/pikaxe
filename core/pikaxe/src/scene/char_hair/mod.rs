mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

#[milo]
pub struct CharHair {
    pub stiffness: f32,
    pub torsion: f32,
    pub inertia: f32,
    pub gravity: f32,

    pub weight: f32,
    pub friction: f32,

    pub min_slack: f32,
    pub max_slack: f32,

    pub strands: Vec<CharHairStrand>,

    pub simulate: bool,
    pub wind: String,
}

#[derive(Default)]
pub struct CharHairStrand {
    pub root: String,
    pub angle: f32,
    pub points: Vec<CharHairPoint>,
    pub base_mat: Matrix3, // Usually the same
    pub root_mat: Matrix3,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum CollideType {
    kCollidePlane,
    kCollideSphere,
    kCollideInsideSphere,
    kCollideCylinder,
    kCollideInsideCylinder
}

impl Default for CollideType {
    fn default() -> Self {
        CollideType::kCollideCylinder
    }
}

impl From<u32> for CollideType {
    fn from(num: u32) -> CollideType {
        match num {
            0 => CollideType::kCollidePlane,
            1 => CollideType::kCollideSphere,
            2 => CollideType::kCollideInsideSphere,
            3 => CollideType::kCollideCylinder,
            4 => CollideType::kCollideInsideCylinder,
            // Default
            _ => CollideType::default(),
        }
    }
}

// TODO: Implement impl From<CollideType> for usize

#[derive(Default)]
pub struct CharHairPoint {
    pub unknown_floats: Vector3,
    pub bone: String,

    pub length: f32,
    pub collide_type: CollideType,
    pub collision: String,

    pub distance: f32,
    pub align_dist: f32,
}

impl Default for CharHair {
    fn default() -> CharHair {
        // TODO: Match default values to C++ code
        CharHair {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // CharHair object
            stiffness: 0.04,
            torsion: 0.1,
            inertia: 0.6,
            gravity: 1.0,

            weight: 1.0,
            friction: 0.3,

            min_slack: 0.0,
            max_slack: 0.0,

            strands: Vec::new(),

            simulate: true,
            wind: String::default(),
        }
    }
}