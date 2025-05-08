mod io;

use pikaxe_macros::*;
use pikaxe_traits::scene::*;
pub use io::*;

#[milo]
pub struct CharLipSync {
    pub visemes: Vec<String>,
    pub frames_count: usize,
    pub data: Vec<u8>
}

impl Default for CharLipSync {
    fn default() -> CharLipSync {
        CharLipSync {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // CharLipSync object
            visemes: Vec::new(),
            frames_count: 0,
            data: Vec::new()
        }
    }
}

pub struct VisemeFrame<'a> {
    pub visemes: Vec<(&'a str, u8)>, // Viseme name, weight
}

impl CharLipSync {
    pub fn get_frames<'a>(&'a self) -> Vec<VisemeFrame> {
        let mut frames = Vec::new();

        let mut idx = 0;
        let mut data = self.data.iter();

        // Interpret frames from raw data
        while let Some(weight_count) = data.next().map(|d| *d as usize) {
            // Check in case data extends beyond frame count
            if idx >= self.frames_count {
                break;
            }

            let mut weights = Vec::new();

            for _ in 0..weight_count {
                let Some(viseme_idx) = data.next().map(|d| *d as usize) else {
                    break;
                };

                let Some(weight) = data.next().map(|d| *d) else {
                    break;
                };

                weights.push((self.visemes[viseme_idx].as_str(), weight));
            }

            frames.push(VisemeFrame {
                visemes: weights
            });

            idx += 1;
        }

        frames
    }
}