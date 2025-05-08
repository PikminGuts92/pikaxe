pub fn calculate_texture_height(tex_size: usize, width: u32, bpp: u32) -> u32 {
    ((tex_size * 8) / (width * bpp) as usize) as u32
}

pub fn read_as_u16(data: &[u8]) -> u16 {
    (data[0] as u16) | (data[1] as u16) << 8
}

pub fn read_as_u16_be(data: &[u8]) -> u16 {
    (data[1] as u16) | (data[0] as u16) << 8
}

pub fn read_as_u24(data: &[u8]) -> u32 {
    (data[0] as u32) | (data[1] as u32) << 8 | (data[2] as u32) << 16
}

pub fn read_as_u24_be(data: &[u8]) -> u32 {
    (data[2] as u32) | (data[1] as u32) << 8 | (data[0] as u32) << 16
}

pub fn interpolate_alphas(packed: &[u8], alphas: &mut [u8; 8]) {
    let alpha_0 = packed[0];
    let alpha_1 = packed[1];

    if alpha_0 > alpha_1 {
        interpolate_6bit_alphas(alpha_0, alpha_1, alphas);
    } else {
        interpolate_4bit_alphas(alpha_0, alpha_1, alphas);
    }
}

pub fn interpolate_alphas_be(packed: &[u8], alphas: &mut [u8; 8]) {
    let alpha_0 = packed[1];
    let alpha_1 = packed[0];

    if alpha_0 > alpha_1 {
        interpolate_6bit_alphas(alpha_0, alpha_1, alphas);
    } else {
        interpolate_4bit_alphas(alpha_0, alpha_1, alphas);
    }
}

fn interpolate_6bit_alphas(alpha_0: u8, alpha_1: u8, alphas: &mut [u8; 8]) {
    alphas[0] = alpha_0;
    alphas[1] = alpha_1;

    alphas[2] = (((6.0 / 7.0) * alpha_0 as f32) + ((1.0 / 7.0) * alpha_1 as f32)) as u8;
    alphas[3] = (((5.0 / 7.0) * alpha_0 as f32) + ((2.0 / 7.0) * alpha_1 as f32)) as u8;
    alphas[4] = (((4.0 / 7.0) * alpha_0 as f32) + ((3.0 / 7.0) * alpha_1 as f32)) as u8;
    alphas[5] = (((3.0 / 7.0) * alpha_0 as f32) + ((4.0 / 7.0) * alpha_1 as f32)) as u8;
    alphas[6] = (((2.0 / 7.0) * alpha_0 as f32) + ((5.0 / 7.0) * alpha_1 as f32)) as u8;
    alphas[7] = (((1.0 / 7.0) * alpha_0 as f32) + ((6.0 / 7.0) * alpha_1 as f32)) as u8;
}

fn interpolate_4bit_alphas(alpha_0: u8, alpha_1: u8, alphas: &mut [u8; 8]) {
    alphas[0] = alpha_0;
    alphas[1] = alpha_1;

    alphas[2] = (((4.0 / 5.0) * alpha_0 as f32) + ((1.0 / 5.0) * alpha_1 as f32)) as u8;
    alphas[3] = (((3.0 / 5.0) * alpha_0 as f32) + ((2.0 / 5.0) * alpha_1 as f32)) as u8;
    alphas[4] = (((2.0 / 5.0) * alpha_0 as f32) + ((3.0 / 5.0) * alpha_1 as f32)) as u8;
    alphas[5] = (((1.0 / 5.0) * alpha_0 as f32) + ((4.0 / 5.0) * alpha_1 as f32)) as u8;

    alphas[6] = 0;
    alphas[7] = 0xff;
}

pub fn mix_colors_66_33(color_0: &[u8; 4], color_1: &[u8; 4], mixed: &mut [u8; 4]) {
    // m = c0 66% + c1 33%
    for (i, m) in mixed.iter_mut().take(3).enumerate() {
        *m = (((color_0[i] as u16 * 2) + color_1[i] as u16) / 3) as u8;
    }

    // Set alpha to max
    mixed[3] = 0xff;
}

pub fn mix_colors_50_50(color_0: &[u8; 4], color_1: &[u8; 4], mixed: &mut [u8; 4]) {
    // m = c0 50% + c1 50%
    for (i, m) in mixed.iter_mut().enumerate() {
        *m = (((color_0[i] as u16 * 2) + color_1[i] as u16) / 3) as u8;
    }
}

pub fn unpack_rgb565(c: u16, rgba: &mut [u8; 4]) {
    let c = c as u32;

    rgba[0] = ((((c & 0b1111_1000_0000_0000) << 16) | ((c & 0b1110_0000_0000_0000) << 11)) >> 24) as u8;
    rgba[1] = ((((c & 0b0000_0111_1110_0000) << 13) | ((c & 0b0000_0110_0000_0000) <<  7)) >> 16) as u8;
    rgba[2] = ((((c & 0b0000_0000_0001_1111) << 11) | ((c & 0b0000_0000_0001_1100) <<  6)) >>  8) as u8;
    rgba[3] = 0xff;
}

pub fn unpack_alpha_indicies(packed: &[u8], indicies: &mut [u8; 16]) {
    unpack_alpha_indicies_from_bytes(packed[0], packed[1], packed[2], &mut indicies[..8]);
    unpack_alpha_indicies_from_bytes(packed[3], packed[4], packed[5], &mut indicies[8..]);
}

pub fn unpack_alpha_indicies_360(packed: &[u8], indicies: &mut [u8; 16]) {
    unpack_alpha_indicies_from_bytes(packed[1], packed[0], packed[3], &mut indicies[..8]);
    unpack_alpha_indicies_from_bytes(packed[2], packed[5], packed[4], &mut indicies[8..]);
}

fn unpack_alpha_indicies_from_bytes(p0: u8, p1: u8, p2: u8, ind: &mut [u8]) {
    ind[0] =   p0 & 0b00_000_111;
    ind[1] = ( p0 & 0b00_111_000) >> 3;
    ind[2] = ((p0 & 0b11_000_000) >> 6) | ((p1 & 0b00_000_001) << 2);
    ind[3] = ( p1 & 0b00_001_110) >> 1;
    ind[4] = ( p1 & 0b01_110_000) >> 4;
    ind[5] = ((p1 & 0b10_000_000) >> 7) | ((p2 & 0b00_000_011) << 1);
    ind[6] = ( p2 & 0b00_011_100) >> 2;
    ind[7] = ( p2 & 0b11_100_000) >> 5;
}

pub fn unpack_indicies(packed: &[u8], indicies: &mut [u8; 16]) {
    for (i, ind) in indicies.chunks_mut(4).enumerate() {
        ind[0] =  packed[i] & 0b00_00_00_11;
        ind[1] = (packed[i] & 0b00_00_11_00) >> 2;
        ind[2] = (packed[i] & 0b00_11_00_00) >> 4;
        ind[3] = (packed[i] & 0b11_00_00_00) >> 6;
    }
}

pub fn unpack_indicies_360(packed: &[u8], indicies: &mut [u8; 16]) {
    const PROXY_IDX: [usize; 4] = [1, 0, 3, 2];

    for (i, ind) in indicies.chunks_mut(4).enumerate() {
        let i = PROXY_IDX[i];

        ind[0] =  packed[i] & 0b00_00_00_11;
        ind[1] = (packed[i] & 0b00_00_11_00) >> 2;
        ind[2] = (packed[i] & 0b00_11_00_00) >> 4;
        ind[3] = (packed[i] & 0b11_00_00_00) >> 6;
    }
}

pub fn unpack_indicies_be(packed: &[u8], indicies: &mut [u8; 16]) {
    for (i, ind) in indicies.chunks_mut(4).enumerate() {
        ind[3] =  packed[i] & 0b00_00_00_11;
        ind[2] = (packed[i] & 0b00_00_11_00) >> 2;
        ind[1] = (packed[i] & 0b00_11_00_00) >> 4;
        ind[0] = (packed[i] & 0b11_00_00_00) >> 6;
    }
}

pub fn zero_out(color: &mut [u8; 4]) {
    for c in color {
        *c = 0x00;
    }
}

pub fn copy_unpacked_pixels(rgba: &mut [u8], colors: &[&[u8; 4]; 4], indicies: &[u8; 16], x: u32, y: u32, width: u32) {
    let x = x as usize;
    let y = y as usize;
    let w = width as usize;

    for (y_i, y_inds) in indicies.chunks(4).enumerate() {
        for (x_i, ind) in y_inds.iter().enumerate() {
            let rgba_offset = linear_offset(x + x_i, y + y_i, w);
            rgba[rgba_offset..(rgba_offset + 4)].copy_from_slice(colors[*ind as usize]);
        }
    }
}

pub fn copy_unpacked_alphas(rgba: &mut [u8], alphas: &[u8; 8], indicies: &[u8; 16], x: u32, y: u32, width: u32) {
    copy_unpacked_channels(rgba, alphas, indicies, x, y, width, 3);
}

pub fn copy_unpacked_channels(rgba: &mut [u8], channels: &[u8; 8], indicies: &[u8; 16], x: u32, y: u32, width: u32, i: usize) {
    let x = x as usize;
    let y = y as usize;
    let w = width as usize;

    for (y_i, y_inds) in indicies.chunks(4).enumerate() {
        for (x_i, ind) in y_inds.iter().enumerate() {
            let rgba_offset = linear_offset(x + x_i, y + y_i, w);
            rgba[rgba_offset + i] = channels[*ind as usize];
        }
    }
}

pub fn set_channels_value(rgba: &mut [u8], x: u32, y: u32, width: u32, i: usize, value: u8) {
    let x = x as usize;
    let y = y as usize;
    let w = width as usize;

    for by in 0..4 {
        for bx in 0..4 {
            let rgba_offset = linear_offset(x + bx, y + by, w);
            rgba[rgba_offset + i] = value;
        }
    }
}

fn linear_offset(x: usize, y: usize, w: usize) -> usize {
    (y * (w << 2)) + (x << 2)
}

pub fn swap_image_bytes(data: &mut [u8]) {
    use rayon::prelude::*;

    data.par_chunks_exact_mut(8)
        .for_each(|d| {
        let mut tmp: u8;

        tmp = d[0];
        d[0] = d[1];
        d[1] = tmp;

        tmp = d[2];
        d[2] = d[3];
        d[3] = tmp;

        tmp = d[4];
        d[4] = d[5];
        d[5] = tmp;

        tmp = d[6];
        d[6] = d[7];
        d[7] = tmp;
    });
}