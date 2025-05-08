mod io;

pub use io::*;
use pikaxe_macros::*;
use pikaxe_traits::scene::*;

#[derive(Default)]
pub struct SampleData {
    pub encoding: i32,
    pub sample_count: i32,
    pub sample_rate: i32,
    pub unknown: bool,
    pub data: Vec<u8>
}

#[milo]
pub struct SynthSample {
    pub file: String,
    pub looped: bool,
    pub loop_start_sample: i32,
    pub loop_end_sample: i32,
    pub sample_data: SampleData,
}

impl Default for SynthSample {
    fn default() -> SynthSample {
        SynthSample {
            // Base object
            name: String::default(),
            type2: String::default(),
            note: String::default(),

            // SynthSample object
            file: String::default(),
            looped: false,
            loop_start_sample: 0,
            loop_end_sample: -1,
            sample_data: SampleData::default()
        }
    }
}

impl SynthSample {
    #[cfg(feature = "audio")]
    pub fn save_as_xma_vec(&self) -> Option<Vec<u8>> {
        use crate::audio::{
            RiffBuilder,
            XmaWavFormat,
            SPEAKER_STEREO_MONO,
            XMA_BITS_PER_SAMPLE,
            XMA_BYTES_PER_PACKET
        };

        // Always mono for sample data
        const CHANNEL_COUNT: u16 = 1;

        if self.sample_data.encoding != 3 {
            return None;
        }

        let sd = &self.sample_data;

        let fmt = XmaWavFormat {
            // Wav stuff
            n_channels: CHANNEL_COUNT,
            n_samples_per_sec: sd.sample_rate as u32,
            n_avg_bytes_per_sec: sd.sample_rate as u32, // Not sure if it matters. Can probably be calculated...
            n_block_align: (CHANNEL_COUNT * XMA_BITS_PER_SAMPLE) / 8,
            // Xma stuff
            num_streams: CHANNEL_COUNT,
            channel_mask: SPEAKER_STEREO_MONO,
            samples_encoded: sd.sample_count as u32,
            bytes_per_block: 0x10000, // Only seen this value so...
            play_begin: 0,
            play_length: sd.sample_count as u32,
            loop_begin: 0, // Don't worry about loop for now
            loop_length: 0,
            loop_count: 0,
            encoder_version: 4,
            block_count: (sd.data.len() / XMA_BYTES_PER_PACKET) as u16
        };

        let wav = RiffBuilder::new()
            .with_type(b"WAVE")
            .and_chunk(b"fmt ", fmt.into_array().as_slice())
            .and_chunk(b"data", &sd.data)
            .build_to_vec();

        Some(wav)
    }
}