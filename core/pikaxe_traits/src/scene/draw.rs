use super::{MiloObject, Sphere};

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

pub trait Draw : MiloObject {
    fn get_showing(&self) -> bool;
    fn set_showing(&mut self, showing: bool);

    fn get_draw_objects(&self) -> &Vec<String>;
    fn get_draw_objects_mut(&mut self) -> &mut Vec<String>;
    fn set_draw_objects(&mut self, draw_objects: Vec<String>);

    fn get_sphere(&self) -> &Sphere;
    fn get_sphere_mut(&mut self) -> &mut Sphere;
    fn set_sphere(&mut self, sphere: Sphere);

    fn get_draw_order(&self) -> f32;
    fn set_draw_order(&mut self, draw_order: f32);

    fn get_override_include_in_depth_only_pass(&self) -> &OverrideIncludeInDepthOnlyPass;
    fn get_override_include_in_depth_only_pass_mut(&mut self) -> &mut OverrideIncludeInDepthOnlyPass;
    fn set_override_include_in_depth_only_pass(&mut self, override_include: OverrideIncludeInDepthOnlyPass);
}