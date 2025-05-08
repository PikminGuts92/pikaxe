use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use std::error::Error;

fn is_version_supported(version: u32) -> bool {
    match version {
        0 => true, // GH1
        4 => true, // GH2/TBRB
        _ => false
    }
}

impl ObjectReadWrite for AnimObject {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        load_anim(self, &mut reader, info, true)?;
        Ok(())
    }

    fn save(&self, _stream: &mut dyn Stream, _info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

pub(crate) fn load_anim<T: Anim>(anim: &mut T, reader: &mut Box<BinaryStream>, info: &SystemInfo, read_meta: bool) -> Result<(), Box<dyn Error>> {
    let version = reader.read_uint32()?;
    if !is_version_supported(version) {
        // TODO: Switch to custom error
        panic!("Anim version \"{}\" is not supported!", version);
    }

    if read_meta {
        load_object(anim, reader, info)?;
    }

    if version < 4 {
        // Reads anim entries
        let anim_entry_count = reader.read_uint32()?;
        for _ in 0..anim_entry_count {
            // TODO: Collect into struct field
            reader.read_prefixed_string()?;
            reader.seek(SeekFrom::Current(8))?; // Skip 2 floats
        }

        let anim_objects = anim.get_anim_objects_mut();
        anim_objects.clear();

        // Reads anim objects
        let anim_count = reader.read_uint32()?;
        for _ in 0..anim_count {
            anim_objects.push(reader.read_prefixed_string()?);
        }
    } else {
        anim.set_frame(reader.read_float32()?);
        anim.set_rate(reader.read_uint32()?.into());
    }

    Ok(())
}

pub(crate) fn save_anim<T: Anim>(anim: &T, writer: &mut Box<BinaryStream>, info: &SystemInfo, write_meta: bool)  -> Result<(), Box<dyn Error>> {
    // TODO: Get version from system info
    let version = if info.version == 10 { 0 } else { 4 };
    writer.write_uint32(version)?;

    if write_meta {
        save_object(anim, writer, info)?;
    }

    if version < 4 {
        // TODO: Save anim entries and write
        writer.write_uint32(0)?;

        // Write anim objects
        writer.write_uint32(anim.get_anim_objects().len() as u32)?;
        for anim_obj in anim.get_anim_objects() {
            writer.write_prefixed_string(anim_obj)?;
        }
    } else {
        writer.write_float32(anim.get_frame())?;
        writer.write_uint32(*anim.get_rate() as u32)?;
    }

    Ok(())
}