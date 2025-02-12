use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use grim_traits::scene::*;
use thiserror::Error as ThisError;
use std::error::Error;

#[derive(Debug, ThisError)]
pub enum CharClipSamplesReadError {
    #[error("CharClipSamples version of {version} not supported")]
    CharClipSamplesNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
        10 | 11 => true, // GH2/GH2 360
        16 => true, // TBRB/GDRB
         _ => false
    }
}

impl ObjectReadWrite for CharClipSamples {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;

        // If not valid, return unsupported error
        if !is_version_supported(version) {
            return Err(Box::new(CharClipSamplesReadError::CharClipSamplesNotSupported {
                version
            }));
        }

        // Metadata is written for CharClip instead for some reason
        load_char_clip(self, &mut reader, info, true)?;

        if version >= 16 {
            self.some_bool = reader.read_boolean()?;
        }

        if version < 13 {
            // Header + data split between two parts. Use char slip samples version

            // Read headers first
            let (full_bones, full_sample_count) = load_char_bones_samples_header(&mut self.full, &mut reader, version)?;
            let (one_bones, one_sample_count) = load_char_bones_samples_header(&mut self.one, &mut reader, version)?;

            if version > 7 {
                // Read duplicate serialized data Probably milo bug
                // TODO: Write specific function that just skips data instead of read
                let mut cbs = CharBonesSamples::default();
                load_char_bones_samples_header(&mut cbs, &mut reader, version)?;
            }

            // Then read data
            load_char_bones_samples_data(&mut self.full, &mut reader, version, full_bones, full_sample_count)?;
            load_char_bones_samples_data(&mut self.one, &mut reader, version, one_bones, one_sample_count)?;
        } else {
            load_char_bones_samples(&mut self.full, &mut reader, info)?;
            load_char_bones_samples(&mut self.one, &mut reader, info)?;
        }

        if version > 14 {
            // Load bones
            let bone_count = reader.read_uint32()?;

            // TODO: Do something with extra bones values
            for _ in 0..bone_count {
                let _name = reader.read_prefixed_string()?;
                let _weight = reader.read_float32()?;
            }
        }

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        // TODO: Get version from system info
        let version = if !self.full.frames.is_empty() {
            13 // Use newer version if it has frames
        } else {
            11
        };

        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        stream.write_uint32(version)?;
        save_char_clip(self, &mut stream, info, true)?;

        if version >= 16 {
            stream.write_boolean(self.some_bool)?;
        }

        if version < 13 {
            // Write as split parts (headers then data samples)
            save_char_bones_samples_header(&self.full, &mut stream, version)?;
            save_char_bones_samples_header(&self.one, &mut stream, version)?;

            if version > 7 {
                // Write ignored data
                // Seems to always write sample count of full + compression 1 (ignored either way so it doesn't matter)
                // TODO: Convert to static object
                save_char_bones_samples_header(&Default::default(), &mut stream, version)?;
            }

            save_char_bones_samples_data(&self.full, &mut stream)?;
            save_char_bones_samples_data(&self.one, &mut stream)?;
        } else {
            save_char_bones_samples(&self.full, &mut stream, version)?;
            save_char_bones_samples(&self.one, &mut stream, version)?;
        }

        if version > 14 {
            todo!("Writing of extra bone data not currently supported for v15 or above");
        }

        Ok(())
    }
}