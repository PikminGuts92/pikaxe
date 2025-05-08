use crate::io::create_new_file;
use wav::{BitDepth, Header, WAV_FORMAT_PCM, read as wav_read, write as wav_write};

pub struct WavEncoder<'a> {
    data: &'a [i16],
    channels: u16,
    sample_rate: u32
}

impl<'a> WavEncoder<'a> {
    pub fn new(data: &'a [i16], channels: u16, sample_rate: u32) -> Self {
        WavEncoder {
            data,
            channels,
            sample_rate
        }
    }

    pub fn encode_to_file<T: AsRef<std::path::Path>>(&self, out_path: T) -> std::io::Result<()> {
        let mut out_file = create_new_file(out_path).unwrap();

        let header = Header::new(WAV_FORMAT_PCM, self.channels, self.sample_rate, 16);
        let bit_data = self.data.to_owned().into(); // Ugh... I hate this so much...

        wav_write(header, &bit_data, &mut out_file)
    }
}

pub fn open_wav<T: AsRef<std::path::Path>>(wav_path: T) -> std::io::Result<(u32, Vec<Vec<i16>>)> {
    let mut wav_file = std::fs::OpenOptions::new()
        .read(true)
        .open(&wav_path)?;

    let (header, bitdepth) = wav_read(&mut wav_file)?;

    let channel_count = header.channel_count as usize;
    let sample_count = match &bitdepth {
        BitDepth::Sixteen(s) => s.len() / channel_count,
        _ => todo!()
    };

    let mut channels = vec![vec![0i16; sample_count]; channel_count];
    let samples = bitdepth.as_sixteen().unwrap();

    /*let ss = samples
        .chunks(channel_count)
        .enumerate()
        .map(|c| c.iter().enumerate())
        .enumerate()
        .for_each(|()|);*/

    // De-interleave samples
    for (s_idx, cs) in samples.chunks_exact(channel_count).enumerate() {
        for (c_idx, s) in cs.iter().enumerate() {
            channels[c_idx][s_idx] = *s;
        }
    }

    Ok((header.sampling_rate, channels))
}