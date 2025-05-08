use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::ObjectReadWrite;
use crate::texture::{Bitmap, decode_dx_image, decode_tpl_image, encode_dx_image, get_dx_bpp, DXGI_Encoding, TPLEncoding};
use crate::system::{Platform, SystemInfo};
use image::buffer::ConvertBuffer;
use image::{ImageBuffer, ImageEncoder, ImageFormat, open, RgbaImage};

use rayon::prelude::*;
use std::error::Error;
use std::path::Path;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum BitmapError {
    #[error("Unsupported texture encoding of {version}")]
    UnsupportedEncoding {
        version: u32,
    },
    #[error("Unsupported bitmap bpp of {bpp}")]
    UnsupportedBitmapBpp {
        bpp: u8
    },
    #[error("Unsupported resolution of {width}x{height}")]
    UnsupportedResolution {
        width: u16,
        height: u16
    },
}

pub enum Image<'a> {
    FromPath(String),
    FromImageBytes(Vec<u8>),
    FromRGBA {
        rgba: &'a [u8],
        width: u16,
        height: u16,
        mips: u8
    },
}

impl Bitmap {
    pub fn from_image(image: Image, info: &SystemInfo) -> Bitmap {
        // Decode rgba image data
        let rgba_buff; // Hack to get around rust lifetimes...
        let (rgba, width, height, _mips) = match image {
            Image::FromRGBA { rgba, width, height, mips } => {
                (rgba, width, height, mips)
            },
            Image::FromPath(img_path) => {
                let img = open(img_path).unwrap();

                let width = img.width() as u16;
                let height = img.height() as u16;

                rgba_buff = Some(img.into_rgba8().into_vec());
                (rgba_buff.as_ref().map(|b| b.as_slice()).unwrap(), width, height, 0)
            },
            _ => todo!()
        };

        match info.platform {
            Platform::X360 | Platform::PS3 => {
                let is_360 = info.platform.eq(&Platform::X360);
                let mut encoding = DXGI_Encoding::DXGI_FORMAT_BC1_UNORM;

                // Use DXT5 encoding if alpha is used
                if rgba.len() >= 4 && rgba.iter().skip(3).step_by(4).any(|&a| a < u8::MAX) {
                    encoding = DXGI_Encoding::DXGI_FORMAT_BC3_UNORM;
                }

                let (bpp, dx_img_size, bpl) = match encoding {
                    DXGI_Encoding::DXGI_FORMAT_BC1_UNORM => (4, ((width as usize) * (height as usize)) / 2, width / 2),
                    _ => (8, (width as usize) * (height as usize), width)
                };

                let mut dx_img = vec![0u8; dx_img_size];

                // Encode without mip maps for now
                let rgba = &rgba[..(width as usize * height as usize * 4)];
                encode_dx_image(rgba, &mut dx_img, width as u32, encoding, is_360);

                Bitmap {
                    bpp,
                    encoding: encoding as u32,
                    mip_maps: 0,

                    width,
                    height,
                    bpl,

                    raw_data: dx_img
                }
            },
            _ => todo!("Support other platforms")
        }
    }

    pub fn import_from_rgba(&mut self, rgba: &[u8]) {
        let expected_size = self.calc_rgba_size();
        if expected_size.ne(&rgba.len()) {
            todo!("Wrong texture size... should return proper error");
        }
    }

    pub fn from_stream(stream: &mut dyn Stream, info: &SystemInfo) -> Result<Bitmap, Box<dyn Error>> {
        let mut bitmap = Bitmap::new();
        bitmap.load(stream, info).and(Ok(bitmap))
    }

    pub fn unpack_rgba(&self, info: &SystemInfo) -> Result<Vec<u8>, Box<dyn Error>> {
        if self.width == 0 || self.height == 0 {
            return Err(Box::new(BitmapError::UnsupportedResolution {
                width: self.width,
                height: self.height,
            }));
        }

        if info.platform == Platform::PS2 && self.encoding == 3 {
            // Decode PS2 bitmap
            let mut rgba = vec![0u8; self.calc_rgba_size()];
            decode_from_bitmap(self, info, &mut rgba[..])?;

            return Ok(rgba);
        } else if info.platform == Platform::PS3 || info.platform == Platform::X360 {
            // Decode next gen texture
            let dx_enc = match self.encoding {
                 8 => DXGI_Encoding::DXGI_FORMAT_BC1_UNORM,
                24 => DXGI_Encoding::DXGI_FORMAT_BC3_UNORM,
                32 => DXGI_Encoding::DXGI_FORMAT_BC5_UNORM,
                _ => {
                    return Err(Box::new(BitmapError::UnsupportedEncoding {
                        version: self.encoding,
                    }));
                }
            };

            let is_360 = info.platform == Platform::X360;

            let mut rgba = vec![0u8; self.calc_rgba_size()];

            let mut mips = self.mip_maps;
            let mut width = self.width;
            let mut height = self.height;

            let mut start_dxt = 0usize;
            let mut start_rgba = 0usize;

            // Hacky way to decode w/ mip maps
            // TODO: Clean up code
            loop {
                let dxt_size = ((width as usize) * (height as usize) * (self.bpp as usize)) / 8;
                let dxt_img = &self.raw_data.as_slice()[start_dxt..(start_dxt + dxt_size)];

                let rgba_size = (width as usize) * (height as usize) * 4;
                let rgba_img = &mut rgba.as_mut_slice()[start_rgba..(start_rgba + rgba_size)];

                decode_dx_image(dxt_img, rgba_img, width as u32, dx_enc, is_360);

                if mips == 0 {
                    break;
                }

                start_dxt += dxt_size;
                start_rgba += rgba_size;

                mips -= 1;
                width >>= 1;
                height >>= 1;
            }

            return Ok(rgba);
        } else if info.platform == Platform::Wii {
            // Decode wii texture
            let tpl_enc = match self.encoding {
                 72 => TPLEncoding::CMP,
                328 => TPLEncoding::CMP_ALPHA,
                584 => TPLEncoding::CMP, // I think it's the same as 72?
                _ => {
                    return Err(Box::new(BitmapError::UnsupportedEncoding {
                        version: self.encoding,
                    }));
                }
            };

            let mut rgba = vec![0u8; self.calc_rgba_size()];

            let mut mips = self.mip_maps;
            let mut width = self.width;
            let mut height = self.height;

            let mut start_tpl = 0usize;
            let mut start_rgba = 0usize;

            // Hacky way to decode w/ mip maps
            // TODO: Clean up code
            loop {
                let tpl_size = ((width as usize) * (height as usize) * (self.bpp as usize)) / 8;
                let tpl_img = &self.raw_data.as_slice()[start_tpl..(start_tpl + tpl_size)];

                let rgba_size = (width as usize) * (height as usize) * 4;
                let rgba_img = &mut rgba.as_mut_slice()[start_rgba..(start_rgba + rgba_size)];

                decode_tpl_image(tpl_img, rgba_img, width as u32, tpl_enc);
                //decode_dx_image(tpl_img, rgba_img, self.width as u32, DXGI_Encoding::DXGI_FORMAT_BC1_UNORM);

                if mips == 0 {
                    break;
                }

                start_tpl += tpl_size;
                start_rgba += rgba_size;

                mips -= 1;
                width >>= 1;
                height >>= 1;
            }

            return Ok(rgba);
        }

        Err(Box::new(BitmapError::UnsupportedEncoding {
            version: self.encoding,
        }))
    }

    fn calc_rgba_size(&self) -> usize {
        let Bitmap { width: w, height: h, mip_maps: mips, ..} = self;
        calc_rgba_size(*w, *h, *mips)
    }
}

impl ObjectReadWrite for Bitmap {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // 0 = 16-byte header (Amp), 1 = 32-byte header
        // TODO: Validate as 0 or 1
        let byte_1 = reader.read_uint8()?;

        if byte_1 == 0 {
            // Load as amp texture (no mip maps)
            self.bpp = reader.read_uint8()?;
            self.encoding = reader.read_uint16()? as u32;
            self.mip_maps = 0u8;

            self.width = reader.read_uint16()?;
            self.height = reader.read_uint16()?;
            self.bpl = reader.read_uint16()?;

            reader.seek(SeekFrom::Current(6))?; // Skip empty bytes
        } else {
            self.bpp = reader.read_uint8()?;
            self.encoding = reader.read_uint32()?;
            self.mip_maps = reader.read_uint8()?;

            self.width = reader.read_uint16()?;
            self.height = reader.read_uint16()?;
            self.bpl = reader.read_uint16()?;

            reader.seek(SeekFrom::Current(19))?; // Skip empty bytes
        }

        // TODO: Calculate expected data size and verify against actual
        let current_pos = reader.pos();
        let stream_len = reader.len()?;
        let rem_bytes = stream_len - current_pos as usize;

        self.raw_data = reader.read_bytes(rem_bytes)?;

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Figure out if this changes per milo version
        stream.write_boolean(true)?;

        stream.write_uint8(self.bpp)?;
        stream.write_uint32(self.encoding)?;
        stream.write_uint8(self.mip_maps)?;

        stream.write_uint16(self.width)?;
        stream.write_uint16(self.height)?;
        stream.write_uint16(self.bpl)?;

        stream.write_bytes(&[0u8; 19])?;
        stream.write_bytes(&self.raw_data)?;

        Ok(())
    }
}

fn calc_rgba_size(mut w: u16, mut h: u16, mut mips: u8) -> usize {
    let mut size = 0;

    loop {
        size += (w as usize) * (h as usize) * 4;

        if mips == 0 {
            break;
        }

        w >>= 1;
        h >>= 1;
        mips -= 1;
    }

    size
}


pub fn decode_from_bitmap(bitmap: &Bitmap, _info: &SystemInfo, rgba: &mut [u8]) -> Result<(), Box<dyn Error>> {
    let Bitmap { bpp, raw_data: data, .. } = bitmap;

    if *bpp == 4 || *bpp == 8 {
        let mut palette = data[..(1 << (*bpp + 2))].to_owned(); // Takes 1024 bytes for 8bpp and 64 bytes for 4bpp
        let encoded = &data[palette.len()..];
        update_alpha_channels(&mut palette, false);

        let mut i = 0; // Image index
        let mut e = 0; // Encoded index

        if *bpp == 4 {
            // Each byte encodes two colors as palette indices
            let mut p1;
            let mut p2;

            while i < rgba.len() {
                // Palette indices
                p1 = ((encoded[e] & 0x0F) as usize) << 2;
                p2 = ((encoded[e] & 0xF0) as usize) >> 2;

                // Copy colors from palette into rgba array
                rgba[i..(i + 4)].clone_from_slice(&palette[p1..(p1 + 4)]);
                rgba[(i + 4)..(i + 8)].clone_from_slice(&palette[p2..(p2 + 4)]);

                // Increment index
                e += 1;
                i += 8; // 2 pixels
            }
        } else { // 8 bpp
            // Each byte encodes single color as palette index
            rgba
                .par_chunks_exact_mut(4)
                .zip(encoded)
                .for_each(|(pixel, enc)| {
                    // Palette index
                    // Swaps bits 3 and 4 with eachother
                    // Ex: 0110 1011 -> 0111 0011
                    let p1 = (((*enc & 0b1110_0111)
                        | ((*enc & 0b0000_1000) << 1)
                        | ((*enc & 0b0001_0000) >> 1)) as usize) << 2;

                    // Copy color from palette into rgba array
                    pixel.clone_from_slice(&palette[p1..(p1 + 4)]);
                });
        }

    } else {
        return Err(Box::new(BitmapError::UnsupportedBitmapBpp { bpp: bitmap.bpp}));
    }

    Ok(())
}

fn update_alpha_channels(data: &mut [u8], reduce: bool) {
    if reduce {
        // 8-bit -> 7-bit alpha
        for alpha in data.iter_mut().skip(3).step_by(4) {
            *alpha = match *alpha {
                0xFF => 0x80,
                _ => *alpha >> 1
            }
        }
    } else {
        // 7-bit -> 8-bit alpha
        for alpha in data.iter_mut().skip(3).step_by(4) {
            *alpha = match *alpha {
                0x80 ..= 0xFF => 0xFF, // It should max out at 0x80 but just in case
                _ => (*alpha & 0x7F) << 1
            }
        }
    }
}

pub fn write_rgba_to_file(width: u32, height: u32, rgba: &[u8], path: &Path) -> Result<(), Box<dyn Error>> {
    let mut image: RgbaImage = ImageBuffer::new(width, height);
    let mut rgba_idx;
    let mut rgba_pix: [u8; 4] = Default::default();

    for (i, p) in image.pixels_mut().enumerate() {
        rgba_idx = i << 2;
        rgba_pix.clone_from_slice(&rgba[rgba_idx..(rgba_idx + 4)]);

        *p = image::Rgba(rgba_pix);
    }

    image.save(path)?;
    Ok(())
}

pub fn write_rgba_to_vec(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut image: RgbaImage = ImageBuffer::new(width, height);
    let mut rgba_idx;
    let mut rgba_pix: [u8; 4] = Default::default();

    for (i, p) in image.pixels_mut().enumerate() {
        rgba_idx = i << 2;
        rgba_pix.clone_from_slice(&rgba[rgba_idx..(rgba_idx + 4)]);

        *p = image::Rgba(rgba_pix);
    }

    let mut png_data = Vec::new();

    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    encoder.write_image(&image, width, height, image::ExtendedColorType::Rgba8).unwrap();

    Ok(png_data)
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use super::*;

    #[rstest]
    #[case(64, 64, 0, 16384)]
    #[case(64, 64, 2, 21504)]
    #[case(256, 256, 4, 349184)]
    #[case(4096, 4096, 0, 67108864)]
    fn test_calc_rgba_size(#[case] w: u16, #[case] h: u16, #[case] mips: u8, #[case] expected: usize) {
        assert_eq!(expected, calc_rgba_size(w, h, mips));
    }
}