use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use thiserror::Error as ThisError;
use std::error::Error;

#[derive(Debug, ThisError)]
pub enum SynthSampleReadError {
    #[error("SynthSample version of {version} not supported")]
    SynthSampleNotSupported {
        version: u32
    },
    #[error("SampleData version of {version} not supported for SynthSample")]
    SampleDataNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
        5 => true, // GH2 4-song/GH2/GH2 360/TBRB/GDRB
        _ => false
    }
}

fn is_sampledata_version_supported(version: u32) -> bool {
    match version {
        11 => true, // GH2 4-song/GH2/GH2 360
        13 => true, // TBRB/GDRB
         _ => false
    }
}

impl ObjectReadWrite for SynthSample {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;

        // If not valid, return unsupported error
        if !is_version_supported(version) {
            return Err(Box::new(SynthSampleReadError::SynthSampleNotSupported {
                version
            }));
        }

        if version > 1 {
            load_object(self, &mut reader, info)?;
        }

        self.file = reader.read_prefixed_string()?;
        self.looped = reader.read_boolean()?;
        self.loop_start_sample = reader.read_int32()?;
        self.loop_end_sample = if version > 2 { reader.read_int32()? } else { -1 };

        // Read sample data
        let sample_data = &mut self.sample_data;
        let version = reader.read_uint32()?;

        // If not valid, return unsupported error
        if !is_sampledata_version_supported(version) {
            return Err(Box::new(SynthSampleReadError::SampleDataNotSupported {
                version
            }));
        }

        sample_data.encoding = reader.read_int32()?;
        sample_data.sample_count = reader.read_int32()?;
        sample_data.sample_rate = reader.read_int32()?;

        let data_size = reader.read_uint32()?;
        sample_data.unknown = reader.read_boolean()?;
        sample_data.data = reader.read_bytes(data_size as usize)?;

        Ok(())
    }

    fn save(&self, _stream: &mut dyn Stream, _info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}