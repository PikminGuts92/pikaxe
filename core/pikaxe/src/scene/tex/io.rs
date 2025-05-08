use crate::{SystemInfo};
use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::{ObjectReadWrite, Tex, load_object, save_object};
use crate::texture::Bitmap;
use thiserror::Error as ThisError;
use std::error::Error;

#[derive(Debug, ThisError)]
pub enum TexReadError {
    #[error("Tex version of {version} not supported")]
    TexVersionNotSupported {
        version: u32
    },
}

impl Tex {
    // TODO: Add from_hmx_image() function

    fn is_magic_valid(magic: u32, info: &SystemInfo) -> bool {
        match info.version {
            // Amp/GH1
            10 => match magic {
                5 => true, // Amp
                7 => true, // AntiGrav
                8 => true, // GH1
                _ => false
            },
            // GH2
            24 => match magic {
                10 => true,
                _ => false
            },
            // GH2 360/TBRB/GDRB
            25 => match magic {
                10 => true,
                11 => true, // GDRB
                _ => false
            },
            28 => match magic {
                10 => true,
                11 => true, // RB3
                _ => false
            },
            _ => false
        }
    }

    pub fn from_stream(stream: &mut dyn Stream, info: &SystemInfo) -> Result<Tex, Box<dyn Error>> {
        let mut tex = Tex::new();
        tex.load(stream, info).and(Ok(tex))
    }
}

impl ObjectReadWrite for Tex {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let magic = reader.read_uint32()?;

        // If not valid, return unsupported error
        if !Tex::is_magic_valid(magic, info) {
            return Err(Box::new(TexReadError::TexVersionNotSupported {
                version: magic
            }));
        }

        load_object(self, &mut reader, info)?;

        // GDRB encoding
        if magic >= 11 && info.version <= 25 {
            // TODO: Save boolean value
            reader.read_boolean()?;
        }

        self.width = reader.read_uint32()?;
        self.height = reader.read_uint32()?;
        self.bpp = reader.read_uint32()?;

        self.ext_path = reader.read_prefixed_string()?;

        if magic >= 8 {
            self.index_f = reader.read_float32()?
        } else {
            self.index_f = 0.0
        }

        self.index = reader.read_int32()?;

        // RB3 encoding
        if magic >= 11 && info.version > 25 {
            // TODO: Save boolean value
            reader.read_boolean()?;
        }

        self.use_ext_path = if magic != 7 {
            reader.read_boolean()?
        } else {
            // AntiGrav - Interpret 32-bit int as bool 
            let bool_int = reader.read_uint32()?;
            bool_int != 0
        };

        if reader.pos() == reader.len()? as u64 {
            return Ok(());
        }

        self.bitmap = match Bitmap::from_stream(reader.as_mut(), info) {
            Ok(bitmap) => Some(bitmap),
            Err(_) => None,
        };

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 10;

        stream.write_uint32(version)?;

        save_object(self, &mut stream, info)?;

        // GDRB encoding
        if version >= 11 && info.version <= 25 {
            // TODO: Write actual boolean value
            stream.write_boolean(false)?;
        }

        stream.write_uint32(self.width)?;
        stream.write_uint32(self.height)?;
        stream.write_uint32(self.bpp)?;

        stream.write_prefixed_string(&self.ext_path)?;
        stream.write_float32(self.index_f)?;
        stream.write_int32(self.index)?;

        // RB3 encoding
        if version >= 11 && info.version > 25 {
            // TODO: Write actual boolean value
            stream.write_boolean(false)?;
        }

        stream.write_boolean(self.use_ext_path && self.bitmap.is_some())?;

        if let Some(bitmap) = &self.bitmap {
            bitmap.save(stream.as_mut(), info)?;
        }

        Ok(())
    }
}