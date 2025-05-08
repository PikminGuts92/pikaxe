mod io;
pub use io::*;
use crate::dta::{DataArray, RootData};
use pikaxe_macros::*;
use pikaxe_traits::scene::*;

// TODO: Move events to shared file
#[derive(Debug, Default)]
pub struct AnimEventFloat {
    pub value: f32,
    pub pos: f32,
}

#[derive(Debug, Default)]
pub struct AnimEventColor {
    pub value: Color4,
    pub pos: f32,
}

#[derive(Debug, Default)]
pub struct AnimEventObject {
    pub text1: String,
    pub text2: String,
    pub pos: f32,
}

#[derive(Debug, Default)]
pub struct AnimEventBool {
    pub value: bool,
    pub pos: f32,
}

#[derive(Debug, Default)]
pub struct AnimEventQuat {
    pub value: Quat,
    pub pos: f32,
}

#[derive(Debug, Default)]
pub struct AnimEventVector3 {
    pub value: Vector3,
    pub pos: f32,
}

#[derive(Debug, Default)]
pub struct AnimEventSymbol {
    pub text: String,
    pub pos: f32,
}

#[derive(Debug)]
pub enum PropKeysEvents {
    Float(Vec<AnimEventFloat>),
    Color(Vec<AnimEventColor>),
    Object(Vec<AnimEventObject>),
    Bool(Vec<AnimEventBool>),
    Quat(Vec<AnimEventQuat>),
    Vector3(Vec<AnimEventVector3>),
    Symbol(Vec<AnimEventSymbol>),
}

impl Default for PropKeysEvents {
    fn default() -> PropKeysEvents {
        PropKeysEvents::Symbol(Default::default())
    }
}

impl PropKeysEvents {
    pub(crate) fn get_enum_value(&self) -> u32 {
        match self {
            PropKeysEvents::Float(_)   => 0,
            PropKeysEvents::Color(_)   => 1,
            PropKeysEvents::Object(_)  => 2,
            PropKeysEvents::Bool(_)    => 3,
            PropKeysEvents::Quat(_)    => 4,
            PropKeysEvents::Vector3(_) => 5,
            PropKeysEvents::Symbol(_)  => 6
        }
    }

    pub fn len(&self) -> usize {
        match self {
            PropKeysEvents::Float(evs)   => evs.len(),
            PropKeysEvents::Color(evs)   => evs.len(),
            PropKeysEvents::Object(evs)  => evs.len(),
            PropKeysEvents::Bool(evs)    => evs.len(),
            PropKeysEvents::Quat(evs)    => evs.len(),
            PropKeysEvents::Vector3(evs) => evs.len(),
            PropKeysEvents::Symbol(evs)  => evs.len()
        }
    }

    pub(crate) fn from_enum_value(value: u32) -> PropKeysEvents {
        match value {
            0 => PropKeysEvents::Float(Vec::new()),
            1 => PropKeysEvents::Color(Vec::new()),
            2 => PropKeysEvents::Object(Vec::new()),
            3 => PropKeysEvents::Bool(Vec::new()),
            4 => PropKeysEvents::Quat(Vec::new()),
            5 => PropKeysEvents::Vector3(Vec::new()),
            6 => PropKeysEvents::Symbol(Vec::new()),
            _ => panic!("Unsupported prop keys enum value of {value}") // TODO: Use dedicated error?
        }
    }
}

#[derive(Debug, Default)]
pub struct PropKeys {
    pub target: String,
    pub property: Vec<DataArray>,

    pub interpolation: u32, // Some flags?
    pub interp_handler: String,

    pub unknown_enum: u32,
    pub events: PropKeysEvents,
}

#[milo]
#[milo_super(Anim)]
#[derive(Debug)]
pub struct PropAnim {
    pub unknown_toggle: bool, // Used in GDRB
    pub keys: Vec<PropKeys>,
}

impl Default for PropAnim {
    fn default() -> PropAnim {
        PropAnim {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // Anim object
            anim_objects: Vec::new(),
            frame: 0.0,
            rate: AnimRate::default(),

            unknown_toggle: false,
            keys: Vec::new(),
        }
    }
}