use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use thiserror::Error as ThisError;
use std::error::Error;

#[derive(Debug, ThisError)]
pub enum CharLipSyncReadError {
    #[error("CharLipSync version of {version} not supported")]
    CharLipSyncVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
        0 => true,
        _ => false
    }
}

impl ObjectReadWrite for CharLipSync {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;

        // If not valid, return unsupported error
        if !is_version_supported(version) {
            // TODO: Switch to custom error
            return Err(Box::new(CharLipSyncReadError::CharLipSyncVersionNotSupported {
                version
            }));
        }

        load_object(self, &mut reader, info)?;

        // Read visemes
        self.visemes.clear();
        let visemes_count = reader.read_uint32()?;
        for _ in 0..visemes_count {
            let viseme = reader.read_prefixed_string()?;
            self.visemes.push(viseme);
        }

        self.frames_count = reader.read_uint32()? as usize;

        // Read keyframe data
        let data_size = reader.read_uint32()? as usize; 
        self.data = reader.read_bytes(data_size)?;

        Ok(())
    }

    fn save(&self, _stream: &mut dyn Stream, _info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        todo!("Implement save() for CharLipSync")

        /*let mut writer = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 1;
        Ok(())*/
    }
}
