use bevy_ecs::prelude::*;
use pikaxe_derive::{autotrait, milo};
use super::{Object, ObjectComponent, Sphere};


#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum OverrideIncludeInDepthOnlyPass {
    kOverrideIncludeInDepthOnlyPass_None,
    kOverrideIncludeInDepthOnlyPass_Include,
    kOverrideIncludeInDepthOnlyPass_DontInclude,
}

impl Default for OverrideIncludeInDepthOnlyPass {
    fn default() -> OverrideIncludeInDepthOnlyPass {
        OverrideIncludeInDepthOnlyPass::kOverrideIncludeInDepthOnlyPass_None
    }
}

impl From<u32> for OverrideIncludeInDepthOnlyPass {
    fn from(num: u32) -> OverrideIncludeInDepthOnlyPass {
        match num {
            0 => OverrideIncludeInDepthOnlyPass::kOverrideIncludeInDepthOnlyPass_None,
            1 => OverrideIncludeInDepthOnlyPass::kOverrideIncludeInDepthOnlyPass_Include,
            2 => OverrideIncludeInDepthOnlyPass::kOverrideIncludeInDepthOnlyPass_DontInclude,
            // Default
            _ => OverrideIncludeInDepthOnlyPass::kOverrideIncludeInDepthOnlyPass_None,
        }
    }
}

#[derive(Default, Clone, Component)]
#[require(ObjectComponent)]
#[autotrait(extends=Object)]
pub struct DrawComponent {
    pub showing: bool,
    pub draw_objects: Vec<String>,
    pub sphere: Sphere,
    pub draw_order: f32,
    pub override_include_in_depth_only_pass: OverrideIncludeInDepthOnlyPass,
}

#[milo(class_name="Draw")]
#[derive(Default, Clone, Bundle)]
pub struct DrawInstance {
    pub(crate) object: ObjectComponent,
    pub(crate) draw: DrawComponent,
}