use rayon::prelude::*;
use super::*;

struct ValuesPtr(*mut [u8]);

unsafe impl Send for ValuesPtr {}
unsafe impl Sync for ValuesPtr {}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DXGI_Encoding {
    DXGI_FORMAT_BC1_UNORM =  8, // DXT1
    DXGI_FORMAT_BC3_UNORM = 24, // DXT5
    DXGI_FORMAT_BC5_UNORM = 32, // ATI2
}

impl Default for DXGI_Encoding {
    fn default() -> DXGI_Encoding {
        DXGI_Encoding::DXGI_FORMAT_BC3_UNORM
    }
}

impl From<u32> for DXGI_Encoding {
    fn from(num: u32) -> DXGI_Encoding {
        match num {
             8 => DXGI_Encoding::DXGI_FORMAT_BC1_UNORM,
            24 => DXGI_Encoding::DXGI_FORMAT_BC3_UNORM,
            32 => DXGI_Encoding::DXGI_FORMAT_BC5_UNORM,
            // Default
            _ => DXGI_Encoding::DXGI_FORMAT_BC3_UNORM,
        }
    }
}

pub fn decode_dx_image(dx_img: &[u8], rgba: &mut [u8], width: u32, encoding: DXGI_Encoding, is_360: bool) {
    match &encoding {
        DXGI_Encoding::DXGI_FORMAT_BC1_UNORM => decode_dxt1_image(dx_img, rgba, width, is_360),
        DXGI_Encoding::DXGI_FORMAT_BC3_UNORM => decode_dxt5_image(dx_img, rgba, width, is_360),
        DXGI_Encoding::DXGI_FORMAT_BC5_UNORM => decode_ati2_image(dx_img, rgba, width, is_360),
    };
}

pub fn encode_dx_image(rgba: &[u8], dx_img: &mut [u8], width: u32, encoding: DXGI_Encoding, is_360: bool) {
    let height = calculate_texture_height(rgba.len(), width, 32);
    encode_dxt_with_lib(rgba, dx_img, width, height, encoding);

    if is_360 {
        swap_image_bytes(dx_img);
    }
}

fn encode_dxt_with_lib(rgba: &[u8], dx_img: &mut [u8], width: u32, height: u32, encoding: DXGI_Encoding) {
    use texpresso::{Algorithm, ColourWeights, COLOUR_WEIGHTS_PERCEPTUAL, Format, Params};

    let format = match encoding {
        DXGI_Encoding::DXGI_FORMAT_BC1_UNORM => Format::Bc1,
        DXGI_Encoding::DXGI_FORMAT_BC3_UNORM => Format::Bc3,
        DXGI_Encoding::DXGI_FORMAT_BC5_UNORM => Format::Bc5,
    };

    let params = Params {
        algorithm: Algorithm::IterativeClusterFit, // Highest quality
        weights: COLOUR_WEIGHTS_PERCEPTUAL,
        weigh_colour_by_alpha: false,
    };

    // Encode dxt image
    format.compress(rgba, width as usize, height as usize, params, dx_img);
}

fn decode_dxt1_image(dx_img: &[u8], rgba: &mut [u8], width: u32, is_360: bool) {
    let bpp = get_dx_bpp(&DXGI_Encoding::DXGI_FORMAT_BC1_UNORM) as u32;

    // Get block counts
    let block_x = width >> 2;
    let block_size = ((16 * bpp) / 8) as usize;

    let read_u16: fn(&[u8]) -> u16;
    let unpack_ind: fn(&[u8], &mut [u8; 16]);

    if is_360 {
        read_u16 = read_as_u16_be;
        unpack_ind = unpack_indicies_360;
    } else {
        read_u16 = read_as_u16;
        unpack_ind = unpack_indicies;
    }

    let rgba = ValuesPtr(rgba);

    dx_img
        .par_chunks_exact(block_size)
        .enumerate()
        .for_each(|(i, block) | {
            let _ = &rgba;
            let bx = i % block_x as usize;
            let by = i / block_x as usize;

            let x = (bx << 2) as u32;
            let y = (by << 2) as u32;

            let mut color_0 = [0u8; 4];
            let mut color_1 = [0u8; 4];
            let mut color_2 = [0u8; 4];
            let mut color_3 = [0u8; 4];

            let mut indicies = [0u8; 16];

            // Read packed bytes
            let packed_0 = read_u16(&block[..2]);
            let packed_1 = read_u16(&block[2..4]);

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
            unpack_ind(&block[4..8], &mut indicies);

            // Copy colors to pixel data
            let colors = [&color_0, &color_1, &color_2, &color_3];

            unsafe {
                let rgba = &mut *rgba.0;
                copy_unpacked_pixels(rgba, &colors, &indicies, x, y, width);
            }
        });
}

fn decode_dxt5_image(dx_img: &[u8], rgba: &mut [u8], width: u32, is_360: bool) {
    let bpp = get_dx_bpp(&DXGI_Encoding::DXGI_FORMAT_BC3_UNORM) as u32;

    // Get block counts
    let block_x = width >> 2;
    let block_size = ((16 * bpp) / 8) as usize;

    let interp_alphas: fn(&[u8], &mut [u8; 8]);
    let unpack_alphas: fn(&[u8], &mut [u8; 16]);
    let read_u16: fn(&[u8]) -> u16;
    let unpack_ind: fn(&[u8], &mut [u8; 16]);

    if is_360 {
        interp_alphas = interpolate_alphas_be;
        unpack_alphas = unpack_alpha_indicies_360;
        read_u16 = read_as_u16_be;
        unpack_ind = unpack_indicies_360;
    } else {
        interp_alphas = interpolate_alphas;
        unpack_alphas = unpack_alpha_indicies;
        read_u16 = read_as_u16;
        unpack_ind = unpack_indicies;
    }

    let rgba = ValuesPtr(rgba);

    dx_img
        .par_chunks_exact(block_size)
        .enumerate()
        .for_each(|(i, block) | {
            let _ = &rgba;
            let bx = i % block_x as usize;
            let by = i / block_x as usize;

            let x = (bx << 2) as u32;
            let y = (by << 2) as u32;

            let mut color_0 = [0u8; 4];
            let mut color_1 = [0u8; 4];
            let mut color_2 = [0u8; 4];
            let mut color_3 = [0u8; 4];

            let mut alphas = [0u8; 8];

            let mut indicies = [0u8; 16];
            let mut alpha_indicies = [0u8; 16];

            interp_alphas(&block[..2], &mut alphas);
            unpack_alphas(&block[2..8], &mut alpha_indicies);

            // Read packed bytes
            let packed_0 = read_u16(&block[8..10]);
            let packed_1 = read_u16(&block[10..12]);

            // Unpack colors to rgba
            unpack_rgb565(packed_0, &mut color_0);
            unpack_rgb565(packed_1, &mut color_1);

            // Interpolate other colors (4 colors)
            mix_colors_66_33(&color_0, &color_1, &mut color_2);
            mix_colors_66_33(&color_1, &color_0, &mut color_3);

            // Unpack color indicies
            unpack_ind(&block[12..16], &mut indicies);

            unsafe {
                let rgba = &mut *rgba.0;

                // Copy colors to pixel data
                let colors = [&color_0, &color_1, &color_2, &color_3];
                copy_unpacked_pixels(rgba, &colors, &indicies, x, y, width);

                // Copy alphas to pixel data
                copy_unpacked_alphas(rgba, &alphas, &alpha_indicies, x, y, width);
            }
        });
}

fn decode_ati2_image(dx_img: &[u8], rgba: &mut [u8], width: u32, is_360: bool) {
    let bpp = get_dx_bpp(&DXGI_Encoding::DXGI_FORMAT_BC5_UNORM) as u32;

    // Get block counts
    let block_x = width >> 2;
    let block_size = ((16 * bpp) / 8) as usize;

    let interp_normals: fn(&[u8], &mut [u8; 8]);
    let unpack_normals: fn(&[u8], &mut [u8; 16]);

    if is_360 {
        interp_normals = interpolate_alphas_be;
        unpack_normals = unpack_alpha_indicies_360;
    } else {
        interp_normals = interpolate_alphas;
        unpack_normals = unpack_alpha_indicies;
    }

    let rgba = ValuesPtr(rgba);

    dx_img
        .par_chunks_exact(block_size)
        .enumerate()
        .for_each(|(i, block) | {
            let _ = &rgba;
            let bx = i % block_x as usize;
            let by = i / block_x as usize;

            let x = (bx << 2) as u32;
            let y = (by << 2) as u32;

            let mut reds = [0u8; 8];
            let mut greens = [0u8; 8];

            let mut red_indicies = [0u8; 16];
            let mut green_indicies = [0u8; 16];

            // Read reds
            interp_normals(&block[..2], &mut reds);
            unpack_normals(&block[2..8], &mut red_indicies);

            // Read greens
            interp_normals(&block[8..10], &mut greens);
            unpack_normals(&block[10..16], &mut green_indicies);

            unsafe {
                let rgba = &mut *rgba.0;

                // Copy red/green channels to pixel data
                copy_unpacked_channels(rgba, &reds, &red_indicies, x, y, width, 0);
                copy_unpacked_channels(rgba, &greens, &green_indicies, x, y, width, 1);

                // Set blues/alphas to 0xFF
                set_channels_value(rgba, x, y, width, 2, 0xFF);
                set_channels_value(rgba, x, y, width, 3, 0xFF);
            }
        });
}


pub fn get_dx_bpp(encoding: &DXGI_Encoding) -> u8 {
    match encoding {
        DXGI_Encoding::DXGI_FORMAT_BC1_UNORM => 4,
        DXGI_Encoding::DXGI_FORMAT_BC3_UNORM => 8,
        DXGI_Encoding::DXGI_FORMAT_BC5_UNORM => 8,
    }
}