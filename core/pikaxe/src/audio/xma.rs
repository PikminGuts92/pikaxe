use bitstream_io::{BigEndian, BitRead, BitReader, Endianness};
use std::io::{Cursor, Read, Result, SeekFrom, Write};

// https://github.com/hrydgard/minidx9/blob/master/Include/xma2defs.h

pub const XMA_BITS_PER_SAMPLE: u16 = 16;
pub const XMA_BYTES_PER_PACKET: usize = 2048;
pub const XMA_SAMPLES_PER_FRAME: usize = 512;
pub const XMA_SAMPLES_PER_SUBFRAME: usize = 128;

pub const WAVE_FORMAT_XMA2: u16 = 0x166;
pub const SPEAKER_STEREO_MONO: u32 = 0x00000004; // SPEAKER_FRONT_CENTER
pub const SPEAKER_STEREO_STEREO: u32 = 0x00000003; // SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT
pub const SPEAKER_ALL: u32 = 0x80000000;

#[derive(Debug)]
struct PacketInfo {
    pub frame_count: u8,
    pub frame_offset_in_bits: u16,
    pub metadata: u8,
    pub skip_count: u8
}

impl PacketInfo {
    /*pub fn from_u32(n: u32) -> Self {
        Self {
            frame_count: (n >> 26) as u8,
            frame_offset_in_bits
        }
    }*/

    pub fn from_reader<R: Read, E: Endianness>(reader: &mut BitReader<R, E>) -> Result<Self> {
        // 32 bytes
        let frame_count = reader.read::<u8>(6)?;
        let frame_offset_in_bits = reader.read::<u16>(15)?;
        let metadata = reader.read::<u8>(3)?;
        let skip_count = reader.read::<u8>(8)?;

        Ok(Self {
            frame_count,
            frame_offset_in_bits,
            metadata,
            skip_count
        })
    }
}

// TODO: Probably put inside struct as method
pub fn read_frame_length<R: Read, E: Endianness>(reader: &mut BitReader<R, E>) -> Result<u16> {
    reader.read::<u16>(15)
}

#[cfg(feature = "audio_experimental")]
pub fn decode_xma_packets(packets: &[u8], _sample_count: i32) -> Result<Vec<i16>> {
    let mut reader = BitReader::endian(Cursor::new(packets), BigEndian);

    let packet = PacketInfo::from_reader(&mut reader)?;
    println!("{packet:#?}");

    let frame_length = read_frame_length(&mut reader)? - 15;
    println!("Frame length: {frame_length}");

    reader.seek_bits(SeekFrom::Current(frame_length as i64))?;

    let frame_length = read_frame_length(&mut reader)? - 15;
    println!("Frame length: {frame_length}");

    todo!()
}

pub struct XmaWavFormat {
    // Traditional wav data
    // w_format_tag (constant)
    pub n_channels: u16,
    pub n_samples_per_sec: u32,
    pub n_avg_bytes_per_sec: u32,
    pub n_block_align: u16, // channels * wBitsPerSample / 8
    // w_bits_per_sample (Always 16 for XMA)
    // cb_size (Size in bytes of the rest of this structure (34))

    // Xma stuff
    pub num_streams: u16,
    pub channel_mask: u32, // SPEAKER
    pub samples_encoded: u32,
    pub bytes_per_block: u32, // 0x10000?
    pub play_begin: u32,
    pub play_length: u32, // Sample length...
    pub loop_begin: u32,
    pub loop_length: u32,
    pub loop_count: u8, // 255 = infinite
    pub encoder_version: u8, // 4 = PG
    pub block_count: u16,
}

impl XmaWavFormat {
    pub fn into_array(&self) -> [u8; 52] {
        let mut data = Cursor::new([0u8; 52]);

        // Write wav data
        data.write_all(&WAVE_FORMAT_XMA2.to_le_bytes()).unwrap();
        data.write_all(&self.n_channels.to_le_bytes()).unwrap();
        data.write_all(&self.n_samples_per_sec.to_le_bytes()).unwrap();
        data.write_all(&self.n_avg_bytes_per_sec.to_le_bytes()).unwrap();
        data.write_all(&self.n_block_align.to_le_bytes()).unwrap();
        data.write_all(&XMA_BITS_PER_SAMPLE.to_le_bytes()).unwrap();
        data.write_all(&34u16.to_le_bytes()).unwrap(); // Size of xma data

        // Write xma data
        data.write_all(&self.num_streams.to_le_bytes()).unwrap();
        data.write_all(&self.channel_mask.to_le_bytes()).unwrap();
        data.write_all(&self.samples_encoded.to_le_bytes()).unwrap();
        data.write_all(&self.bytes_per_block.to_le_bytes()).unwrap();
        data.write_all(&self.play_begin.to_le_bytes()).unwrap();
        data.write_all(&self.play_length.to_le_bytes()).unwrap();
        data.write_all(&self.loop_begin.to_le_bytes()).unwrap();
        data.write_all(&self.loop_length.to_le_bytes()).unwrap();
        data.write_all(&self.loop_count.to_le_bytes()).unwrap();
        data.write_all(&self.encoder_version.to_le_bytes()).unwrap();
        data.write_all(&self.block_count.to_le_bytes()).unwrap();

        data.into_inner()
    }
}

pub struct RiffBuilder<'a> {
    riff_type: Option<&'a [u8; 4]>,
    chunks: Vec<(&'a [u8; 4], &'a [u8])>
}

impl<'a> RiffBuilder<'a> {
    pub fn new() -> Self {
        Self {
            riff_type: None,
            chunks: Vec::new()
        }
    }

    pub fn with_type(mut self, typ: &'a [u8; 4]) -> Self {
        self.riff_type = Some(typ);
        self
    }

    pub fn and_chunk(mut self, id: &'a [u8; 4], data: &'a [u8]) -> Self {
        self.chunks.push((id, data));
        self
    }

    pub fn build_to_vec(self) -> Vec<u8> {
        let total_chunk_size = self.calc_total_chunk_size() as u32;

        let mut writer = Cursor::new(Vec::new());

        // Write header data
        writer.write_all(b"RIFF").unwrap();
        writer.write_all(&total_chunk_size.to_le_bytes()).unwrap();
        if let Some(typ) = self.riff_type {
            writer.write_all(typ).unwrap();
        }

        // Write chunk data
        for (id, data) in self.chunks.iter() {
            writer.write_all(*id).unwrap();
            writer.write_all(&(data.len() as u32).to_le_bytes()).unwrap();
            writer.write_all(data).unwrap();
        }

        writer.into_inner()
    }

    fn calc_total_chunk_size(&self) -> usize {
        self.riff_type
            .as_ref()
            .map(|t| t.len())
            .unwrap_or_default() +
            self.chunks
                .iter()
                .map(|(id, d)| id.len() + d.len() + 4)
                .sum::<usize>()
    }
}