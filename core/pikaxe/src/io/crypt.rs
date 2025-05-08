use std::io::{Read, Seek, Write};

pub fn crypt_dtb_style(data: &mut [u8], key: i32, xor: Option<u8>) {
    let xor = xor.unwrap_or(0);
    let mut k = key;

    for b in data.iter_mut() {
        k = apply_xor(k);
        *b = *b ^ (k as u8) ^ xor;
    }
}

fn apply_xor(v: i32) -> i32 {
    let mut v = ((v - ((v / 0x1F31D) * 0x1F31D)) * 0x41A7) - ((v / 0x1F31D) * 0xB14);

    if v <= 0 {
        v += 0x7FFFFFFF
    }

    v
}