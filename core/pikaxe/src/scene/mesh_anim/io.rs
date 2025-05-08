use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use thiserror::Error as ThisError;
use std::error::Error;

#[derive(Debug, ThisError)]
pub enum MeshAnimReadError {
    #[error("MeshAnim version of {version} not supported")]
    MeshAnimVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
        1 => true,
        _ => false
    }
}

impl ObjectReadWrite for MeshAnim {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;

        // If not valid, return unsupported error
        if !is_version_supported(version) {
            // TODO: Switch to custom error
            return Err(Box::new(MeshAnimReadError::MeshAnimVersionNotSupported {
                version
            }));
        }

        if version >= 1 {
            load_object(self, &mut reader, info)?;
        }
        load_anim(self, &mut reader, info, false)?;

        self.mesh = reader.read_prefixed_string()?;

        // Reset keys
        self.vert_point_keys.clear();
        self.vert_text_keys.clear();
        self.vert_color_keys.clear();

        self.vert_point_keys = load_keys(&mut reader, load_vector3)?;
        self.vert_text_keys = load_keys(&mut reader, load_vector2)?;
        self.vert_color_keys = load_keys(&mut reader, load_color4)?;

        self.keys_owner = reader.read_prefixed_string()?;

        Ok(())
    }

    fn save(&self, _stream: &mut dyn Stream, _info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        todo!("Implement save() for MeshAnim")

        /*let mut writer = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 1;
        Ok(())*/
    }
}

fn load_keys<T: std::fmt::Debug + Default>(reader: &mut Box<BinaryStream>, loader: impl Fn(&mut T, &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>>) -> Result<Vec<AnimEvent<Vec<T>>>, Box<dyn Error>> {
    let count = reader.read_uint32()?;
    let mut keys = Vec::new();

    for _ in 0..count {
        let value_count = reader.read_uint32()?;
        let mut values = Vec::new();

        for _ in 0..value_count {
            let mut value = T::default();
            loader(&mut value, reader)?;

            values.push(value);
        }

        let pos = reader.read_float32()?;

        keys.push(AnimEvent {
            value: values,
            pos
        })
    }

    Ok(keys)
}