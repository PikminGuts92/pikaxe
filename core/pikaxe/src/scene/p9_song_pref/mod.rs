mod io;
pub use io::*;
use pikaxe_macros::*;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum P9SongPrefCharacter {
    TBRB_George,
    TBRB_John,
    TBRB_Paul,
    TBRB_Ringo,
    GDRB_Mike,
    GDRB_Billie,
    GDRB_Tre
}

impl From<P9SongPrefCharacter> for usize {
    fn from(val: P9SongPrefCharacter) -> Self {
        match val {
            P9SongPrefCharacter::TBRB_George | P9SongPrefCharacter::GDRB_Mike => 0,
            P9SongPrefCharacter::TBRB_John | P9SongPrefCharacter::GDRB_Billie => 1,
            P9SongPrefCharacter::TBRB_Paul => 2,
            P9SongPrefCharacter::TBRB_Ringo | P9SongPrefCharacter::GDRB_Tre => 3
        }
    }
}

#[milo]
#[derive(Debug, Default)]
pub struct P9SongPref {
    pub venue: String,
    pub minivenues: Vec<String>,
    pub scenes: Vec<String>,
    // TODO: Add scene groups?

    pub dreamscape_outfit: String,
    pub studio_outfit: String,

    pub instruments: [Vec<String>; 4],
    pub tempo: String,
    pub song_clips: String,
    pub dreamscape_font: String,

    // TBRB
    pub george_amp: String,
    pub john_amp: String,
    pub paul_amp: String,
    pub mixer: String,

    pub dreamscape_camera: String,
    pub lyric_part: String,

    // GDRB
    pub normal_outfit: String,
    pub bonus_outfit: String,
    pub drum_set: String,
    pub era: String,
    pub cam_directory: String,
    pub media_directory: String,
    pub song_intro_cam: String,
    pub win_cam: String,
}

impl P9SongPref {
    pub fn get_instruments(&self, idx: usize) -> Option<&Vec<String>> {
        self.instruments.get(idx)
    }

    pub fn get_instruments_by_character(&self, char: P9SongPrefCharacter) -> &Vec<String> {
        &self.instruments[usize::from(char)]
    }
}