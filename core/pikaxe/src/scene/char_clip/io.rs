use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use log::warn;
use thiserror::Error as ThisError;
use std::error::Error;

#[derive(Debug, ThisError)]
pub enum CharClipReadError {
    #[error("CharClip version of {version} not supported")]
    CharClipVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
         5 => true, // GH2/GH2 360
        12 => true, // TBRB/GDRB
         _ => false
    }
}

pub(crate) fn load_char_clip<T: CharClip>(char_clip: &mut T, reader: &mut Box<BinaryStream>, info: &SystemInfo, read_meta: bool) -> Result<(), Box<dyn Error>> {
    let version = reader.read_uint32()?;

    // If not valid, return unsupported error
    if !is_version_supported(version) {
        // TODO: Switch to custom error
        return Err(Box::new(CharClipReadError::CharClipVersionNotSupported {
            version
        }));
    }

    if read_meta {
        load_object(char_clip, reader, info)?;
    }

    char_clip.set_start_beat(reader.read_float32()?);
    char_clip.set_end_beat(reader.read_float32()?);
    char_clip.set_beats_per_sec(reader.read_float32()?);

    char_clip.set_flags(reader.read_uint32()?);
    char_clip.set_play_flags(reader.read_uint32()?);

    char_clip.set_blend_width(reader.read_float32()?);

    if version > 3 {
        char_clip.set_range(reader.read_float32()?);
    }

    if version == 5 {
        let unknown = reader.read_boolean()?;
        warn!("Skipping unknown bool with value {unknown} at offset 0x{:X} for CharClip with version {version}", reader.pos());
    } else if version > 5 {
        char_clip.set_relative(reader.read_prefixed_string()?);
    }

    /*if (version - 9) < 2 {
        todo!()
    }*/

    if version > 9 {
        char_clip.set_unknown_1(reader.read_int32()?);
    }

    if version > 11 {
        char_clip.set_do_not_decompress(reader.read_boolean()?);
    }

    let node_count = if version < 8 {
        reader.read_uint32()?
    } else {
        reader.seek(SeekFrom::Current(4))?; // Skip node size
        reader.read_uint32()?
    };

    // Read nodes
    let mut nodes = Vec::new();

    for _ in 0..node_count {
        let name = reader.read_prefixed_string()?;
        let value_count = reader.read_uint32()?;

        let mut values = Vec::new();

        for _ in 0..value_count {
            let frame = reader.read_float32()?;
            let weight = reader.read_float32()?;

            values.push(ClipNodeData {
                frame,
                weight
            });
        }

        nodes.push(ClipNode {
            name,
            values
        });
    }

    char_clip.set_nodes(nodes);

    if version < 3 {
        todo!()
    }

    if version < 7 {
        // Deprecated fields
        let enter_event = reader.read_prefixed_string()?;
        let exit_event = reader.read_prefixed_string()?;

        if !enter_event.is_empty() {
            warn!("Found value \"{enter_event}\" for enter_event at offset 0x{:X} for CharClip with version {version}", reader.pos());
        }

        if !exit_event.is_empty() {
            warn!("Found value \"{exit_event}\" for exit_event at offset 0x{:X} for CharClip with version {version}", reader.pos());
        }
    }

    // Read events
    let event_count = reader.read_uint32()?;
    let mut events = Vec::new();

    for _ in 0..event_count {
        let frame = reader.read_float32()?;
        let script = reader.read_prefixed_string()?;

        events.push(FrameEvent {
            frame,
            script
        });
    }

    char_clip.set_events(events);

    Ok(())
}

pub(crate) fn save_char_clip<T: CharClip>(char_clip: &T, writer: &mut Box<BinaryStream>, info: &SystemInfo, write_meta: bool) -> Result<(), Box<dyn Error>> {
    let version = 5;
    writer.write_uint32(version)?;

    if write_meta {
        save_object(char_clip, writer, info)?;
    }

    writer.write_float32(char_clip.get_start_beat())?;
    writer.write_float32(char_clip.get_end_beat())?;
    writer.write_float32(char_clip.get_beats_per_sec())?;

    writer.write_uint32(char_clip.get_flags())?;
    writer.write_uint32(char_clip.get_play_flags())?;

    writer.write_float32(char_clip.get_blend_width())?;
    writer.write_float32(char_clip.get_range())?;

    writer.write_boolean(false)?; // Unknown bool, always false

    // Write nodes
    writer.write_uint32(char_clip.get_nodes().len() as u32)?;
    for node in char_clip.get_nodes() {
        writer.write_prefixed_string(&node.name)?;
        writer.write_uint32(node.values.len() as u32)?;

        for value in node.values.iter() {
            writer.write_float32(value.frame)?;
            writer.write_float32(value.weight)?;
        }
    }

    // Writes empty data for deprecated enter/exit events
    writer.write_uint64(0)?;

    // Write events
    writer.write_uint32(char_clip.get_events().len() as u32)?;
    for event in char_clip.get_events() {
        writer.write_float32(event.frame)?;
        writer.write_prefixed_string(&event.script)?;
    }

    Ok(())
}