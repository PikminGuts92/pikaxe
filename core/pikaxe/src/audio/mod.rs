mod adpcm;
mod str;
mod vgs;
mod wav;
mod xma;

pub use self::adpcm::*;
pub use self::str::*;
pub use self::vgs::*;
pub use self::wav::*;
pub(crate) use self::xma::*;

pub fn convert_to_samples(data: Vec<u8>) -> Vec<i16> {
    // Wait to stabilize https://github.com/rust-lang/rust/issues/74985
    /*data
        .as_chunks()
        .map(|d: [u8; 2]| i16::from_le_bytes(d))
        .collect::<Vec<_>>()*/

    let mut buffer = vec![0i16; data.len() / 2];

    for (i, d) in data.chunks_exact(std::mem::size_of::<i16>()).enumerate() {
        let v = i16::from_le_bytes([d[0], d[1]]);

        buffer[i] = v;
    }

    buffer
}