use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use std::collections::HashSet;
use std::error::Error;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum GroupLoadError {
    #[error("Group version {version} is not supported")]
    GroupVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
         4 => true, // Amp
         7 => true, // GH1
        11 => true, // GH2 4-song
        12 => true, // GH2/GH2 360
        13 => true, // RB1
        14 => true, // TBRB/GDRB
        _ => false
    }
}

impl ObjectReadWrite for GroupObject {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;
        if !is_version_supported(version) {
            return Err(Box::new(GroupLoadError::GroupVersionNotSupported {
                version
            }));
        }

        load_object(self, &mut reader, info)?;
        load_anim(self, &mut reader, info, false)?;
        load_trans(self, &mut reader, info, false)?;
        load_draw(self, &mut reader, info, false)?;

        self.objects.clear();
        if version > 10 {
            let object_count = reader.read_uint32()?;
            for _ in 0..object_count {
                self.objects.push(reader.read_prefixed_string()?);
            }

            self.environ = reader.read_prefixed_string()?;
        } else {
            // Copy anim/draw/trans objects from legacy version
            let mut obj_strings = HashSet::new();

            for anim in self.get_anim_objects() {
                obj_strings.insert(anim.to_owned());
            }

            for draw in self.get_draw_objects() {
                obj_strings.insert(draw.to_owned());
            }

            for trans in self.get_trans_objects() {
                obj_strings.insert(trans.to_owned());
            }

            for obj in obj_strings {
                self.objects.push(obj);
            }
        }

        if version > 11 {
            self.lod = reader.read_prefixed_string()?;
            self.lod_screen_size = reader.read_float32()?;
        } else if version == 4 {
            reader.seek(SeekFrom::Current(4))?;

            let object_count = reader.read_uint32()?;
            for _ in 0..object_count {
                self.objects.push(reader.read_prefixed_string()?);
            }

            // Unknown - Matches group name
            reader.read_prefixed_string()?;

            // Zero'd numbers
            reader.seek(SeekFrom::Current(8))?;
        } else if version == 7 {
            // Unknown - Matches group name
            reader.read_prefixed_string()?;
        }

        /*if version <= 12 {
            let lod_width = reader.read_float32()?;
            let lod_height = reader.read_float32()?;

            // Calculate ratio
            if lod_height != 0.0 {
                self.lod_screen_size = lod_width / lod_height;
            } else {
                self.lod_screen_size = 0.0;
            }
        }*/

        if version > 13 {
            self.sort_in_world = reader.read_boolean()?;
        }

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 12;

        stream.write_uint32(version)?;

        save_object(self, &mut stream, info)?;
        save_anim(self, &mut stream, info, false)?;
        save_trans(self, &mut stream, info, false)?;
        save_draw(self, &mut stream, info, false)?;

        if version >= 11 {
            // Write objects
            stream.write_uint32(self.get_objects().len() as u32)?;
            for obj in self.get_objects() {
                stream.write_prefixed_string(obj)?;
            }
        }

        // Write environ
        stream.write_prefixed_string(&self.environ)?;

        if version == 11 {
            return Ok(());
        }

        if version <= 12 {
            // Write ratio
            // TODO: Figure out how to extract from computed lod_screen_size
            stream.write_float32(0.0)?; // Width
            stream.write_float32(0.0)?; // Height
        } else {
            stream.write_prefixed_string(&self.draw_only)?;
            stream.write_prefixed_string(&self.lod)?;
            stream.write_float32(self.lod_screen_size)?;

            if version >= 14 {
                stream.write_boolean(self.sort_in_world)?;
            }
        }

        Ok(())
    }
}