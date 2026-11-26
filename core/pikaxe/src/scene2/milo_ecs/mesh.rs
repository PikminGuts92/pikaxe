use bevy_ecs::prelude::*;
use pikaxe_derive::{autotrait, milo};
use super::{Draw, DrawComponent, Object, ObjectComponent, Trans, TransComponent, Transform, Vector4};


#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Volume {
    kVolumeEmpty,
    kVolumeTriangles,
    kVolumeBSP,
    kVolumeBox,
}

impl Default for Volume {
    fn default() -> Volume {
        Volume::kVolumeEmpty
    }
}

impl From<u32> for Volume {
    fn from(num: u32) -> Volume {
        match num {
            0 => Volume::kVolumeEmpty,
            1 => Volume::kVolumeTriangles,
            2 => Volume::kVolumeBSP,
            3 => Volume::kVolumeBox,
            // Default
            _ => Volume::kVolumeEmpty,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Mutable {
    kMutableNone = 0,
    kMutableVerts = 31,
    kMutableFaces = 32,
    kMutableAll = 63,
}

impl Default for Mutable {
    fn default() -> Mutable {
        Mutable::kMutableNone
    }
}

impl From<u32> for Mutable {
    fn from(num: u32) -> Mutable {
        match num {
             0 => Mutable::kMutableNone,
            31 => Mutable::kMutableVerts,
            32 => Mutable::kMutableFaces,
            63 => Mutable::kMutableAll,
            // Default
            _ => Mutable::kMutableNone,
        }
    }
}

#[derive(Default, Clone)]
pub struct BoneTrans {
    pub name: String,
    pub trans: Transform,
}

#[derive(Default, Clone)]
pub struct UV {
    pub u: f32,
    pub v: f32,
}

#[derive(Clone)]
pub struct Vert {
    pub pos: Vector4,
    pub normals: Vector4,
    pub uv: UV,
    pub bones: [u16; 4],
    pub weights: [f32; 4],
    pub tangent: Vector4,
}

impl Default for Vert {
    fn default() -> Vert {
        Vert {
            pos: Vector4::default(),
            normals: Vector4::default(),
            uv: UV::default(),
            bones: [0, 1, 2, 3],
            weights: [1.0, 1.0, 1.0, 1.0],
            tangent: Vector4 {
                // TODO: Verify this is correct
                x:  1.0,
                y:  0.0,
                z:  0.0,
                w: -1.0,
            }
        }
    }
}


#[derive(Default, Clone, Component)]
#[require(ObjectComponent)]
#[autotrait(extends=Trans + Draw)]
pub struct MeshComponent {
    pub mat: String,
    pub geom_owner: String,

    pub mutable: Mutable,
    pub volume: Volume,

    pub vertices: Vec<Vert>,
    pub faces: Vec<[u16; 3]>,

    pub face_groups: Vec<u8>,
    pub bones: Vec<BoneTrans>,
    pub keep_mesh_data: bool,
    pub exclude_from_self_shadow: bool,
    pub has_ao_calculation: bool,
}

#[milo(class_name="Mesh")]
#[derive(Default, Clone, Bundle)]
pub struct MeshInstance {
    pub(crate) object: ObjectComponent,
    pub(crate) draw: DrawComponent,
    pub(crate) trans: TransComponent,
}