use super::*;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum TPLEncoding {
    RGBA8     =  64, //  6
    CMP       =  72, // 14
    CMP_ALPHA = 328,
}

impl Default for TPLEncoding {
    fn default() -> TPLEncoding {
        TPLEncoding::CMP
    }
}

impl From<u32> for TPLEncoding {
    fn from(num: u32) -> TPLEncoding {
        match num {
             64 => TPLEncoding::RGBA8,
             72 => TPLEncoding::CMP,
            328 => TPLEncoding::CMP_ALPHA,
            // Default
            _ => TPLEncoding::CMP,
        }
    }
}

pub fn decode_tpl_image(tpl_img: &[u8], rgba: &mut [u8], width: u32, encoding: TPLEncoding) {
    match &encoding {
        TPLEncoding::RGBA8 => todo!("Implement wii RGBA8 texture decoding"),
        TPLEncoding::CMP => decode_cmp_image(tpl_img, rgba, width),
        TPLEncoding::CMP_ALPHA => decode_cmp_alpha_image(tpl_img, rgba, width),
    };
}

fn decode_cmp_image(tpl_img: &[u8], rgba: &mut [u8], width: u32) {
    let bpp = get_tpl_bpp(&TPLEncoding::CMP);

    // Get block counts
    let block_x = width >> 3;
    let block_y = calculate_texture_height(tpl_img.len(), width, bpp) >> 3;
    let block_size = ((16 * bpp) / 8) as usize;

    let mut packed_0;
    let mut packed_1;

    let mut color_0 = [0u8; 4];
    let mut color_1 = [0u8; 4];
    let mut color_2 = [0u8; 4];
    let mut color_3 = [0u8; 4];

    let mut indices = [0u8; 16];

    let mut i = 0usize; // Block index
    let mut x;
    let mut y;

    for by in 0..block_y {
        for bx in 0..block_x {
            for t in 0..4 {
                // TODO: Probably refactor
                // Hacky way of adding 2x2 groups
                x = (bx << 3) + ((t & 1) << 2);
                y = (by << 3) + ((t & 2) << 1);

                // Read packed bytes (Wii is reverse endian compared to DXT)
                packed_0 = read_as_u16_be(&tpl_img[i..(i + 2)]);
                packed_1 = read_as_u16_be(&tpl_img[(i + 2)..(i + 4)]);

                // Unpack colors to rgba
                unpack_rgb565(packed_0, &mut color_0);
                unpack_rgb565(packed_1, &mut color_1);

                // Interpolate other colors
                if packed_0 > packed_1 {
                    // 4 colors
                    mix_colors_66_33(&color_0, &color_1, &mut color_2);
                    mix_colors_66_33(&color_1, &color_0, &mut color_3);
                } else {
                    // 3 colors + transparent
                    mix_colors_50_50(&color_0, &color_1, &mut color_2);
                    zero_out(&mut color_3);
                }

                // Unpack color indicies
                unpack_indicies_be(&tpl_img[(i + 4)..(i + 8)], &mut indices);

                // Copy colors to pixel data
                let colors = [&color_0, &color_1, &color_2, &color_3];
                copy_unpacked_pixels(rgba, &colors, &indices, x, y, width);

                i += block_size;
            }
        }
    }
}

fn decode_cmp_alpha_image(tpl_img: &[u8], rgba: &mut [u8], width: u32) {
    let bpp = get_tpl_bpp(&TPLEncoding::CMP_ALPHA);

    // Get block counts
    let block_x = width >> 3;
    let block_y = calculate_texture_height(tpl_img.len(), width, bpp) >> 3;
    let block_size = ((16 * bpp) / 8) as usize;

    let mut packed_0;
    let mut packed_1;

    let mut color_0 = [0u8; 4];
    let mut color_1 = [0u8; 4];
    let mut color_2 = [0u8; 4];
    let mut color_3 = [0u8; 4];

    let mut indices = [0u8; 16];

    let mut i = 0usize; // Block index
    let mut x;
    let mut y;

    for by in 0..block_y {
        for bx in 0..block_x {
            for t in 0..4 {
                // TODO: Probably refactor
                // Hacky way of adding 2x2 groups
                x = (bx << 3) + ((t & 1) << 2);
                y = (by << 3) + ((t & 2) << 1);

                // Read packed bytes (Wii is reverse endian compared to DXT)
                packed_0 = read_as_u16_be(&tpl_img[i..(i + 2)]);
                packed_1 = read_as_u16_be(&tpl_img[(i + 2)..(i + 4)]);

                // Unpack colors to rgba
                unpack_rgb565(packed_0, &mut color_0);
                unpack_rgb565(packed_1, &mut color_1);

                // Interpolate other colors
                if packed_0 > packed_1 {
                    // 4 colors
                    mix_colors_66_33(&color_0, &color_1, &mut color_2);
                    mix_colors_66_33(&color_1, &color_0, &mut color_3);
                } else {
                    // 3 colors + transparent
                    mix_colors_50_50(&color_0, &color_1, &mut color_2);
                    zero_out(&mut color_3);
                }

                // Unpack color indicies
                unpack_indicies_be(&tpl_img[(i + 4)..(i + 8)], &mut indices);

                // Copy colors to pixel data
                let colors = [&color_0, &color_1, &color_2, &color_3];
                copy_unpacked_pixels(rgba, &colors, &indices, x, y, width);

                // Skip alphas for now
                // Not sure why this works...
                i += block_size >> 1;
            }
        }
    }
}

fn get_tpl_bpp(encoding: &TPLEncoding) -> u32 {
    match encoding {
        TPLEncoding::RGBA8     => 32,
        TPLEncoding::CMP       =>  4,
        TPLEncoding::CMP_ALPHA =>  8,
    }
}