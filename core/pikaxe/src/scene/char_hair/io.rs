use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use std::error::Error;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum CharHairLoadError {
    #[error("CharHair version {version} is not supported")]
    CharHairVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
         2 => true, // GH2/GH2 360
        //11 => true, // TBRB
        _ => false
    }
}

impl ObjectReadWrite for CharHair {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;
        if !is_version_supported(version) {
            return Err(Box::new(CharHairLoadError::CharHairVersionNotSupported {
                version
            }));
        }

        load_object(self, &mut reader, info)?;

        self.stiffness = reader.read_float32()?;
        self.torsion = reader.read_float32()?;
        self.inertia = reader.read_float32()?;
        self.gravity = reader.read_float32()?;

        self.weight = reader.read_float32()?;
        self.friction = reader.read_float32()?;

        if version >= 11 {
            self.min_slack = reader.read_float32()?;
            self.max_slack = reader.read_float32()?;
        }

        // Read strands
        let strand_count = reader.read_uint32()?;
        for _ in 0..strand_count {
            let mut strand = CharHairStrand::default();

            strand.root = reader.read_prefixed_string()?;
            strand.angle = reader.read_float32()?;

            // Read points
            let point_count = reader.read_uint32()?;
            for _ in 0..point_count {
                let mut point = CharHairPoint::default();

                load_vector3(&mut point.unknown_floats, &mut reader)?;
                point.bone = reader.read_prefixed_string()?;

                point.length = reader.read_float32()?;
                point.collide_type = reader.read_uint32()?.into();
                point.collision = reader.read_prefixed_string()?;

                point.distance = reader.read_float32()?;
                point.align_dist = reader.read_float32()?;

                strand.points.push(point);
            }

            // Read rotation + scale matrices
            load_matrix3(&mut strand.base_mat, &mut reader)?;
            load_matrix3(&mut strand.root_mat, &mut reader)?;

            self.strands.push(strand);
        }

        self.simulate = reader.read_boolean()?;

        if version >= 11 {
            self.wind = reader.read_prefixed_string()?;
        }

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 2;

        stream.write_uint32(version)?;

        save_object(self, &mut stream, info)?;

        todo!("Support for writing CharHair not implemented yet");
        //Ok(())
    }
}