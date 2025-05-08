use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use std::collections::HashSet;
use std::error::Error;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum CharBoneLoadError {
    #[error("CharBone version {version} is not supported")]
    CharBoneVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
         2 => true, // GH2
         3 => true, // GH2 360
         8 => true, // TBRB
        _ => false
    }
}

impl ObjectReadWrite for CharBone {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;
        if !is_version_supported(version) {
            return Err(Box::new(CharBoneLoadError::CharBoneVersionNotSupported {
                version
            }));
        }

        load_object(self, &mut reader, info)?;
        load_trans(self, &mut reader, info, false)?;

        //Ok(())
        todo!()
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 3;

        stream.write_uint32(version)?;

        save_object(self, &mut stream, info)?;
        save_trans(self, &mut stream, info, false)?;

        stream.write_boolean(self.position)?;
        stream.write_boolean(self.scale)?;

        stream.write_uint32(self.rotation as u32)?;
        if version < 5 {
            stream.write_uint32(RotationConstraint::kRotNone as u32)?;
        }

        if version >= 3 {
            stream.write_float32(self.unknown)?;
        }

        Ok(())
    }
}