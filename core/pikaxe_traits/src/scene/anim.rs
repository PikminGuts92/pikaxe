use super::MiloObject;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum AnimRate {
    k30_fps,
    k480_fpb,
    k30_fps_ui,
    k1_fpb,
    k30_fps_tutorial
}

impl Default for AnimRate {
    fn default() -> AnimRate {
        AnimRate::k30_fps
    }
}

impl From<u32> for AnimRate {
    fn from(num: u32) -> AnimRate {
        match num {
            0 => AnimRate::k30_fps,
            1 => AnimRate::k480_fpb,
            2 => AnimRate::k30_fps_ui,
            3 => AnimRate::k1_fpb,
            4 => AnimRate::k30_fps_tutorial,
            // Default
            _ => AnimRate::default(),
        }
    }
}

pub trait Anim : MiloObject {
    fn get_anim_objects(&self) -> &Vec<String>;
    fn get_anim_objects_mut(&mut self) -> &mut Vec<String>;
    fn set_anim_objects(&mut self, anim_objects: Vec<String>);

    fn get_frame(&self) -> f32;
    fn set_frame(&mut self, frame: f32);

    fn get_rate(&self) -> &AnimRate;
    fn get_rate_mut(&mut self) -> &mut AnimRate;
    fn set_rate(&mut self, rate: AnimRate);
}