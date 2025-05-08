use crate::io::*;
use std::io::Error as IOError;
use thiserror::Error as ThisError;

const VGS_MAGIC: &[u8; 4] = b"VgS!";
const VGS_CHANNEL_COUNT: usize = 15;

const VAG_BYTES_PER_BLOCK: usize = 16;
const VAG_SAMPLES_PER_BLOCK: usize = 28;

const VAG_FILTERS: [[f64; 2]; 5] = [
    [0., 0.],
    [ (60. / 64.), 0.],
    [ (115. / 64.), (-52. / 64.) ],
    [ (98. / 64.), (-55. / 64.) ],
    [ (122. / 64.), (-60. / 64.) ],
];

#[derive(Default)]
pub struct VgsChannelInfo {
    pub sample_rate: u32, // Usually 32k, 44.1k, or 48k
    pub block_count: u32
}

pub struct VgsFile {
    pub version: u32, // 2
    pub channels: [VgsChannelInfo; VGS_CHANNEL_COUNT], // Not sure if other versions support different #
    pub data: Vec<u8>,
}

impl Default for VgsFile {
    fn default() -> Self {
        Self {
            version: 2,
            channels: Default::default(),
            data: Vec::new(),
        }
    }
}

#[derive(Debug, ThisError)]
pub enum VgsReadError {
    #[error("Can't read file")]
    UnknownReadError,
    #[error("Unrecognized magic value")]
    InvalidMagic,
    #[error("Unsupported version of {version}")]
    UnsupportedVersion { version: u32 },
    #[error("IO Error")]
    IO { io_error: IOError }
}

impl From<IOError> for VgsReadError {
    fn from(value: IOError) -> Self {
        VgsReadError::IO {
            io_error: value
        }
    }
}

impl VgsFile {
    pub fn from_data(data: &[u8]) -> Result<VgsFile, VgsReadError> {
        let mut stream = MemoryStream::from_slice_as_read(data);
        let mut reader = BinaryStream::from_stream_with_endian(&mut stream, IOEndian::Little);

        // Read magic
        let magic = reader.read_bytes(4).map_err(|_| VgsReadError::UnknownReadError)?;
        if magic.ne(VGS_MAGIC) {
            return Err(VgsReadError::InvalidMagic);
        }

        // Read version
        let version = reader.read_uint32().map_err(|_| VgsReadError::UnknownReadError)?;
        if ![2].iter().any(|v| v.eq(&version)) {
            return Err(VgsReadError::UnsupportedVersion { version });
        }

        let mut vgs = VgsFile {
            version,
            ..Default::default()
        };

        // Read channel info
        // TODO: Validate sample rate + block count is same or zero'd
        for ch_info in vgs.channels.iter_mut() {
            let sam_rate = reader.read_uint32().map_err(|_| VgsReadError::UnknownReadError)?;
            let block_count = reader.read_uint32().map_err(|_| VgsReadError::UnknownReadError)?;

            ch_info.sample_rate = sam_rate;
            ch_info.block_count = block_count;
        }

        // Read samples
        let stream_size: usize = vgs
            .channels
            .iter()
            .filter(|ch| ch.sample_rate > 0 && ch.block_count > 0)
            .map(|ch| ch.block_count as usize * VAG_BYTES_PER_BLOCK)
            .sum();

        vgs.data = reader.read_bytes(stream_size).map_err(|_| VgsReadError::UnknownReadError)?;

        Ok(vgs)
    }

    pub fn from_reader<T: std::io::Read + std::io::Seek>(reader: &mut T) -> Result<VgsFile, VgsReadError> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        Self::from_data(&data)
    }

    pub fn get_channel_count(&self) -> usize {
        self
            .channels
            .iter()
            .filter(|ch| ch.block_count > 0)
            .count()
    }

    pub fn get_sample_rate(&self) -> u32 {
        self
            .channels
            .iter()
            .map(|ch| ch.sample_rate)
            .max()
            .unwrap() // Shouldn't fail...
    }

    pub fn decode_samples_as_channels(&self) -> Vec<Vec<i16>> {
        let channel_count = self.get_channel_count();
        let mut channel_data = Vec::new();

        let mut decoder = VAGDecoder::new();

        for VgsChannelInfo { sample_rate, block_count } in self.channels.iter() {
            if sample_rate.eq(&0) || block_count.eq(&0) {
                continue;
            }

            let mut decoded_data = Vec::new();
            let ch_idx = channel_data.len();

            for block_idx in 0..(*block_count as usize) {
                let start_idx = (block_idx * channel_count * VAG_BYTES_PER_BLOCK) + (ch_idx * VAG_BYTES_PER_BLOCK);
                let end_idx = start_idx + VAG_BYTES_PER_BLOCK;

                let block = &self.data[start_idx..end_idx];
                let decoded_samples = decoder.decode_block(block);

                decoded_samples.into_iter().for_each(|s| decoded_data.push(s));
            }

            channel_data.push(decoded_data);
        }

        channel_data
    }
}

pub struct VAGDecoder {
    state: (f64, f64)
}

impl VAGDecoder {
    pub fn new() -> Self {
        VAGDecoder {
            state: (0., 0.)
        }
    }

    pub fn reset(&mut self) {
        self.state = (0., 0.);
    }

    pub fn decode_block(&mut self, block: &[u8]) -> [i16; VAG_SAMPLES_PER_BLOCK] {
        let (ref mut s0, ref mut s1) = self.state;

        let mut predictor = high_nibble(block[0]) as usize;
        let shift = low_nibble(block[0]);
        //let flags = block[1];

        if predictor > 4 {
            // Shouldn't happen?
            predictor = 0;
        }

        let mut out_samples = [0i16; VAG_SAMPLES_PER_BLOCK];

        for (i, b) in block.iter().skip(2).enumerate() {
            out_samples[i * 2] = ((low_nibble(*b) as i16) << 12) >> shift;
            out_samples[(i * 2) + 1] = ((high_nibble(*b) as i16) << 12) >> shift;
        }

        for s in out_samples.iter_mut() {
            let filt = (*s as f64) + (*s0 * VAG_FILTERS[predictor][0]) + (*s1 * VAG_FILTERS[predictor][1]);
            *s1 = *s0;
            *s0 = filt;

            *s = quantize(filt);
        }

        out_samples
    }
}

fn high_nibble(n: u8) -> u8 {
    (n >> 4) & 15
}

fn low_nibble(n: u8) -> u8 {
    n & 15
}

fn quantize(s: f64) -> i16 {
    match (s + 0.5) as i32 {
        n if n > i16::MAX as i32 => i16::MAX,
        n if n < i16::MIN as i32 => i16::MIN,
        n @ _ => n as i16
    }
}