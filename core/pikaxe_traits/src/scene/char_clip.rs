use super::MiloObject;

#[derive(Default)]
pub struct ClipNode {
    pub name: String,
    pub values: Vec<ClipNodeData>,
}

#[derive(Default)]
pub struct ClipNodeData {
    pub frame: f32,
    pub weight: f32,
}

#[derive(Default)]
pub struct FrameEvent {
    pub frame: f32,
    pub script: String,
}

pub trait CharClip : MiloObject {
    fn get_start_beat(&self) -> f32;
    fn set_start_beat(&mut self, start_beat: f32);

    fn get_end_beat(&self) -> f32;
    fn set_end_beat(&mut self, end_beat: f32);

    fn get_beats_per_sec(&self) -> f32;
    fn set_beats_per_sec(&mut self, beats_per_sec: f32);

    fn get_flags(&self) -> u32;
    fn set_flags(&mut self, flags: u32);

    fn get_play_flags(&self) -> u32;
    fn set_play_flags(&mut self, play_flags: u32);

    fn get_blend_width(&self) -> f32;
    fn set_blend_width(&mut self, blend_width: f32);

    fn get_range(&self) -> f32;
    fn set_range(&mut self, range: f32);

    fn get_relative(&self) -> &String;
    fn get_relative_mut(&mut self) -> &mut String;
    fn set_relative(&mut self, relative: String);

    fn get_unknown_1(&self) -> i32;
    fn set_unknown_1(&mut self, unknown_1: i32);

    fn get_do_not_decompress(&self) -> bool;
    fn set_do_not_decompress(&mut self, do_not_decompress: bool);

    fn get_nodes(&self) -> &Vec<ClipNode>;
    fn get_nodes_mut(&mut self) -> &mut Vec<ClipNode>;
    fn set_nodes(&mut self, nodes: Vec<ClipNode>);

    fn get_events(&self) -> &Vec<FrameEvent>;
    fn get_events_mut(&mut self) -> &mut Vec<FrameEvent>;
    fn set_events(&mut self, events: Vec<FrameEvent>);
}