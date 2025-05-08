mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

use super::{CharBone4Bone, CharBonesSamples};

#[milo]
pub struct CharClipSamples {
    // TODO: Move CharClip fields to macros
    // Start CharClip
    pub start_beat: f32,
    pub end_beat: f32,
    pub beats_per_sec: f32,

    pub flags: u32,
    pub play_flags: u32,

    pub blend_width: f32,
    pub range: f32,
    pub relative: String,

    pub unknown_1: i32,
    pub do_not_decompress: bool,

    pub nodes: Vec<ClipNode>,
    pub events: Vec<FrameEvent>,
    // End CharClip

    pub some_bool: bool,
    pub full: CharBonesSamples,
    pub one: CharBonesSamples,
    pub bones: Vec<CharBone4Bone>,
}

impl Default for CharClipSamples {
    fn default() -> CharClipSamples {
        CharClipSamples {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // CharClip object
            start_beat: 0.0,
            end_beat: 0.0,
            beats_per_sec: 0.0,

            flags: 0,
            play_flags: 0,

            blend_width: 0.0,
            range: 0.0,
            relative: String::new(),

            unknown_1: -1,
            do_not_decompress: false,

            nodes: Vec::new(),
            events: Vec::new(),

            // CharClipSamples object
            some_bool: true,
            full: CharBonesSamples::default(),
            one: CharBonesSamples::default(),
            bones: Vec::new(),
        }
    }
}

impl CharClip for CharClipSamples {
    fn get_start_beat(&self) -> f32 {
        self.start_beat
    }
    fn set_start_beat(&mut self, start_beat: f32) {
        self.start_beat = start_beat;
    }

    fn get_end_beat(&self) -> f32 {
        self.end_beat
    }
    fn set_end_beat(&mut self, end_beat: f32) {
        self.end_beat = end_beat;
    }

    fn get_beats_per_sec(&self) -> f32 {
        self.beats_per_sec
    }
    fn set_beats_per_sec(&mut self, beats_per_sec: f32) {
        self.beats_per_sec = beats_per_sec;
    }

    fn get_flags(&self) -> u32 {
        self.flags
    }
    fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    fn get_play_flags(&self) -> u32 {
        self.play_flags
    }
    fn set_play_flags(&mut self, play_flags: u32) {
        self.play_flags = play_flags;
    }

    fn get_blend_width(&self) -> f32 {
        self.blend_width
    }
    fn set_blend_width(&mut self, blend_width: f32) {
        self.blend_width = blend_width;
    }

    fn get_range(&self) -> f32 {
        self.range
    }
    fn set_range(&mut self, range: f32) {
        self.range = range;
    }

    fn get_relative(&self) -> &String {
        &self.relative
    }
    fn get_relative_mut(&mut self) -> &mut String {
        &mut self.relative
    }
    fn set_relative(&mut self, relative: String) {
        self.relative = relative;
    }

    fn get_unknown_1(&self) -> i32 {
        self.unknown_1
    }
    fn set_unknown_1(&mut self, unknown_1: i32) {
        self.unknown_1 = unknown_1;
    }

    fn get_do_not_decompress(&self) -> bool {
        self.do_not_decompress
    }
    fn set_do_not_decompress(&mut self, do_not_decompress: bool) {
        self.do_not_decompress = do_not_decompress;
    }

    fn get_nodes(&self) -> &Vec<ClipNode> {
        &self.nodes
    }
    fn get_nodes_mut(&mut self) -> &mut Vec<ClipNode> {
        &mut self.nodes
    }
    fn set_nodes(&mut self, nodes: Vec<ClipNode>) {
        self.nodes = nodes;
    }

    fn get_events(&self) -> &Vec<FrameEvent> {
        &self.events
    }
    fn get_events_mut(&mut self) -> &mut Vec<FrameEvent> {
        &mut self.events
    }
    fn set_events(&mut self, events: Vec<FrameEvent>) {
        self.events = events;
    }
}