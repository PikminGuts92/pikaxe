use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use std::error::Error;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum DrawLoadError {
    #[error("Draw version {version} is not supported")]
    DrawVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
        0 => true,     // Freq/Amp Demo/Amp
        1 => true,     // GH1
        3 | 4 => true, // TBRB/GDRB
        _ => false
    }
}

impl ObjectReadWrite for DrawObject {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        load_draw(self, &mut reader, info, true)
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut writer = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        save_draw(self, &mut writer, info, true)
    }
}

pub(crate) fn load_draw<T: Draw>(draw: &mut T, reader: &mut Box<BinaryStream>, info: &SystemInfo, read_meta: bool)  -> Result<(), Box<dyn Error>> {
    let version = reader.read_uint32()?;
    if !is_version_supported(version) {
        return Err(Box::new(DrawLoadError::DrawVersionNotSupported {
            version
        }));
    }

    // Read as null-terminated or prefixed string depending on version
    // Need to check sys_info because freq draw version is same as amp
    let read_string = if info.version <= 6 {
        |reader: &mut Box<BinaryStream>| reader.read_null_terminated_string()
    } else {
        |reader: &mut Box<BinaryStream>| reader.read_prefixed_string()
    };

    if read_meta {
        load_object(draw, reader, info)?;
    }

    draw.set_showing(reader.read_boolean()?);

    if version < 2 {
        let draw_objects = draw.get_draw_objects_mut();
        draw_objects.clear();

        // Reads draw objects
        let draw_count = reader.read_uint32()?;
        for _ in 0..draw_count {
            draw_objects.push(read_string(reader)?);
        }
    }

    if version > 0 {
        load_sphere(draw.get_sphere_mut(), reader)?;
    }

    if version > 2 {
        draw.set_draw_order(reader.read_float32()?);
    }

    if version >= 4 {
        draw.set_override_include_in_depth_only_pass(reader.read_uint32()?.into());
    }

    Ok(())
}

pub(crate) fn save_draw<T: Draw>(draw: &T, writer: &mut Box<BinaryStream>, info: &SystemInfo, write_meta: bool)  -> Result<(), Box<dyn Error>> {
    // TODO: Get version from system info
    let version = 3;
    writer.write_uint32(version)?;

    if write_meta {
        save_object(draw, writer, info)?;
    }

    writer.write_boolean(draw.get_showing())?;
    save_sphere(draw.get_sphere(), writer)?;

    if version >= 3 {
        writer.write_float32(draw.get_draw_order())?;
    }

    if version >= 4 {
        writer.write_uint32(*draw.get_override_include_in_depth_only_pass() as u32)?;
    }

    Ok(())
}