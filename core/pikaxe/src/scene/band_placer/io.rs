use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use std::error::Error;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum BandPlacerLoadError {
    #[error("BandPlacer version {version} is not supported")]
    BandPlacerVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
         2 => true, // GH2/GH2 360
        _ => false
    }
}

impl ObjectReadWrite for BandPlacer {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;
        if !is_version_supported(version) {
            return Err(Box::new(BandPlacerLoadError::BandPlacerVersionNotSupported {
                version
            }));
        }

        load_object(self, &mut reader, info)?;
        load_draw(self, &mut reader, info, false)?;
        load_trans(self, &mut reader, info, false)?;

        self.center = reader.read_prefixed_string()?;

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 2;

        stream.write_uint32(version)?;

        save_object(self, &mut stream, info)?;
        save_trans(self, &mut stream, info, false)?;
        save_draw(self, &mut stream, info, false)?;

        stream.write_prefixed_string(&self.center)?;

        Ok(())
    }
}