use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use thiserror::Error as ThisError;
use std::error::Error;

#[derive(Debug, ThisError)]
pub enum MorphReadError {
    #[error("Morph version of {version} not supported")]
    MorphVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
        3 => true,
        _ => false
    }
}

impl ObjectReadWrite for Morph {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;

        // If not valid, return unsupported error
        if !is_version_supported(version) {
            // TODO: Switch to custom error
            return Err(Box::new(MorphReadError::MorphVersionNotSupported {
                version
            }));
        }

        if version > 3 {
            load_object(self, &mut reader, info)?;
        }
        load_anim(self, &mut reader, info, false)?;

        let pose_count = reader.read_uint32()?;
        reader.seek(SeekFrom::Current(4))?; // Skip 0 data

        // Read poses
        self.poses.clear();
        for _ in 0..pose_count {
            let pose = load_morph_pose(&mut reader)?;
            self.poses.push(pose);
        }

        self.normals = reader.read_boolean()?;
        self.spline = reader.read_boolean()?;
        self.intensity = reader.read_float32()?;

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut writer = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 3;
        writer.write_uint32(version)?;

        if version > 3 {
            save_object(self, &mut writer, info)?;
        }
        save_anim(self, &mut writer, info, false)?;

        writer.write_uint32(self.poses.len() as u32)?;
        writer.write_uint32(0)?; // Always 0

        for pose in self.poses.iter() {
            save_morph_pose(pose, &mut writer)?;
        }

        writer.write_boolean(self.normals)?;
        writer.write_boolean(self.spline)?;
        writer.write_float32(self.intensity)?;

        Ok(())
    }
}

fn load_morph_pose(reader: &mut Box<BinaryStream>) -> Result<MorphPose, Box<dyn Error>> {
    let count = reader.read_uint32()?;
    reader.seek(SeekFrom::Current(4))?; // Skip 0 data

    let mut keys = Vec::new();

    for _ in 0..count {
        let pos = reader.read_float32()?;
        let value = reader.read_float32()?;

        keys.push(AnimEvent {
            value,
            pos
        })
    }

    Ok(MorphPose {
        events: keys
    })
}

fn save_morph_pose(pose: &MorphPose, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_uint32(pose.events.len() as u32)?;
    writer.write_uint32(0)?; // Always 0

    for ev in pose.events.iter() {
        writer.write_float32(ev.pos)?;
        writer.write_float32(ev.value)?;
    }

    Ok(())
}