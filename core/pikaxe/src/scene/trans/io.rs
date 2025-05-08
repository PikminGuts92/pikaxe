use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use std::error::Error;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum TransLoadError {
    #[error("Trans version {version} is not supported")]
    TransVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
        5 => true, // Freq/Amp/Amp Demo
        8 => true, // GH1
        9 => true, // GH2/RB1/RB2/TBRB/GDRB
        _ => false
    }
}

impl ObjectReadWrite for TransObject {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        load_trans(self, &mut reader, info, true)
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut writer = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        save_trans(self, &mut writer, info, true)
    }
}

pub(crate) fn load_trans<T: Trans>(trans: &mut T, reader: &mut Box<BinaryStream>, info: &SystemInfo, read_meta: bool)  -> Result<(), Box<dyn Error>> {
    let version = reader.read_uint32()?;
    if !is_version_supported(version) {
        return Err(Box::new(TransLoadError::TransVersionNotSupported {
            version
        }));
    }

    // Read as null-terminated or prefixed string depending on version
    // Need to check sys_info because freq trans version is same as amp
    let read_string = if info.version <= 6 {
        |reader: &mut Box<BinaryStream>| reader.read_null_terminated_string()
    } else {
        |reader: &mut Box<BinaryStream>| reader.read_prefixed_string()
    };

    if read_meta {
        load_object(trans, reader, info)?;
    }

    load_matrix(trans.get_local_xfm_mut(), reader)?;
    load_matrix(trans.get_world_xfm_mut(), reader)?;

    if version < 9 {
        let trans_objects = trans.get_trans_objects_mut();
        trans_objects.clear();

        // Reads trans objects
        let trans_count = reader.read_uint32()?;
        for _ in 0..trans_count {
            trans_objects.push(read_string(reader)?);
        }
    }

    // Read constraint
    if version > 6 {
        trans.set_constraint(reader.read_uint32()?.into());
    } else if version == 6 {
        _ = reader.read_uint32()?;
        trans.set_constraint(TransConstraint::kConstraintNone);
    } else if version < 3 {
        if version > 0 {
            _ = reader.read_uint32()?;
        }

        trans.set_constraint(TransConstraint::kConstraintNone);
    } else { // 3, 4, 5
        _ = reader.read_uint32()?; // Some flags
    }

    if version < 7 {
        // Skip 3 ints
        reader.seek(SeekFrom::Current(12))?;
    }

    if version < 5 {
        // Skip bool
        reader.seek(SeekFrom::Current(1))?;
    }

    if version < 2 {
        // Skip vector4
        reader.seek(SeekFrom::Current(16))?;
    }

    if version > 5 {
        trans.set_target(reader.read_prefixed_string()?);
    } else {
        trans.set_target(String::new());
    }

    if version > 6 {
        trans.set_preserve_scale(reader.read_boolean()?);
    } else {
        trans.set_preserve_scale(true);
    }

    if version < 9 {
        if version >= 7 {
            trans.set_parent(reader.read_prefixed_string()?);
        } else {
            trans.set_parent(String::new());
        }
    } else {
        trans.set_parent(reader.read_prefixed_string()?);
    }

    Ok(())
}

pub(crate) fn save_trans<T: Trans>(trans: &T, writer: &mut Box<BinaryStream>, info: &SystemInfo, write_meta: bool)  -> Result<(), Box<dyn Error>> {
    // TODO: Get version from system info
    let version = 9;
    writer.write_uint32(version)?;

    if write_meta {
        save_object(trans, writer, info)?;
    }

    save_matrix(trans.get_local_xfm(), writer)?;
    save_matrix(trans.get_world_xfm(), writer)?;

    writer.write_uint32(*trans.get_constraint() as u32)?;
    writer.write_prefixed_string(trans.get_target())?;

    writer.write_boolean(trans.get_preserve_scale())?;
    writer.write_prefixed_string(trans.get_parent())?;

    Ok(())
}