use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use std::error::Error;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum CamLoadError {
    #[error("Cam version {version} is not supported")]
    CamVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
         9 => true, // GH1
        12 => true, // GH2/TBRB
        _ => false
    }
}

impl ObjectReadWrite for CamObject {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;
        if !is_version_supported(version) {
            return Err(Box::new(CamLoadError::CamVersionNotSupported {
                version
            }));
        }

        load_object(self, &mut reader, info)?;
        load_trans(self, &mut reader, info, false)?;

        if version < 10 {
            load_draw(self, &mut reader, info, false)?;
        }

        self.near_plane = reader.read_float32()?;
        self.far_plane = reader.read_float32()?;
        self.y_fov = reader.read_float32()?;

        load_rect(&mut self.screen_rect, &mut reader)?;
        load_vector2(&mut self.z_range, &mut reader)?;
        self.target_tex = reader.read_prefixed_string()?;

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 12;

        stream.write_uint32(version)?;

        save_object(self, &mut stream, info)?;
        save_trans(self, &mut stream, info, false)?;

        if version < 10 {
            save_draw(self, &mut stream, info, false)?;
        }

        stream.write_float32(self.near_plane)?;
        stream.write_float32(self.far_plane)?;
        stream.write_float32(self.y_fov)?;

        save_rect(&self.screen_rect, &mut stream)?;
        save_vector2(&self.z_range, &mut stream)?;
        stream.write_prefixed_string(&self.target_tex)?;

        Ok(())
    }
}