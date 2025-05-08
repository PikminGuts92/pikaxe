use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use std::error::Error;
use thiserror::Error as ThisError;
use log::info;

#[derive(Debug, ThisError)]
pub enum ColorPaletteLoadError {
    #[error("ColorPalette version {version} is not supported")]
    ColorPaletteVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
         1 => true, // RB1 and up, I don't think a v2 exists
        _ => false
    }
}

impl ObjectReadWrite for ColorPalette {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;
        if !is_version_supported(version) {
            return Err(Box::new(ColorPaletteLoadError::ColorPaletteVersionNotSupported {
                version
            }));
        }

        load_object(self, &mut reader, info)?;

        self.num_colors = reader.read_uint32()?;
        self.colors = load_palette_colors(self.num_colors, &mut reader)?;

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 1;

        stream.write_uint32(version)?;

        save_object(self, &mut stream, info)?;

        stream.write_uint32(self.num_colors)?;
        
        for color in &self.colors {
            stream.write_float32(color.red)?;
            stream.write_float32(color.green)?;
            stream.write_float32(color.blue)?;
            stream.write_float32(color.unknown)?;
        }

        Ok(())
    }
}

fn load_palette_colors(num_colors: u32, reader: &mut Box<BinaryStream>,) -> Result<Vec<PaletteColor>, Box<dyn Error>> {
    let mut colors = Vec::new();

    for _ in 0..num_colors {
        let red = reader.read_float32()?;
        let green = reader.read_float32()?;
        let blue = reader.read_float32()?;
        let unknown = reader.read_float32()?;

        colors.push(PaletteColor {red, green, blue, unknown, })
    }

    Ok(colors)
}