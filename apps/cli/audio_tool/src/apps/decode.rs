use crate::apps::SubApp;
use pikaxe::{Platform, SystemInfo};
use pikaxe::audio::*;
use pikaxe::io::{FileStream, IOEndian};
use pikaxe::scene::{ObjectReadWrite, SampleData, SynthSample};

use clap::Parser;
use std::error::Error;
use std::io::{Read, Seek, Write};
use std::fs;
use std::path::{Path, PathBuf};

enum FileType {
    Vgs,
    Str,
    SynthSample(u32, IOEndian)
}

#[derive(Parser, Debug)]
pub struct DecoderApp {
    #[arg(help = "Path to input audio (.vgs, .str, SynthSample (Xbox 360))", required = true)]
    pub input_path: String,
    #[arg(help = "Path to output audio (.wav, .xma (Only for Xbox 360 samples))", required = true)]
    pub output_path: String,
}

impl SubApp for DecoderApp {
    fn process(&mut self) -> Result<(), Box<dyn Error>> {
        let input_path = Path::new(&self.input_path);
        let output_path = Path::new(&self.output_path);

        let input_type = match input_path.extension().and_then(|e| e.to_str()) {
            Some(ext) if "str".eq_ignore_ascii_case(ext) => FileType::Str,
            Some(ext) if "vgs".eq_ignore_ascii_case(ext) => FileType::Vgs,
            Some(ext) => {
                // Guess file type from binary structure
                if let Some(file_type) = guess_type_from_magic(input_path) {
                    file_type
                } else {
                    println!("Input audio with extension \".{ext}\" is not supported");
                    return Ok(());
                }
            },
            _ => {
                // Guess file type from binary structure
                if let Some(file_type) = guess_type_from_magic(input_path) {
                    file_type
                } else {
                    println!("Input audio not supported");
                    return Ok(());
                }
            }
        };

        let input_type_name = match input_type {
            FileType::Str => "STR",
            FileType::SynthSample(_, _) => "SynthSample",
            FileType::Vgs => "VGS",
        };

        println!("Detected input file type of \"{input_type_name}\"");

        let _output_ext = match (&input_type, output_path.extension().and_then(|e| e.to_str())) {
            (FileType::Str, Some(ext)) if "wav".eq_ignore_ascii_case(ext) => ext,
            (FileType::SynthSample(_, _), Some(ext)) if "xma".eq_ignore_ascii_case(ext) => ext,
            (FileType::Vgs, Some(ext)) if "wav".eq_ignore_ascii_case(ext) => ext,
            (_, Some(ext)) => {
                println!("Output audio with extension \".{ext}\" is not supported");
                return Ok(());
            },
            _ => {
                println!("Output audio not supported");
                return Ok(());
            }
        };

        match input_type {
            FileType::Str => {
                // Decode (returns interleaved audio samples)
                println!("Decoding...");
                let sample_data = decode_str_file(input_path)?;

                // Encode to wav
                let encoder = WavEncoder::new(&sample_data, 2, 48_000);
                encoder.encode_to_file(output_path).map_err(|e| Box::new(e) as Box<dyn Error>)?;
            },
            FileType::SynthSample(version, endian) => {
                let sys_info = SystemInfo {
                    version,
                    platform: Platform::X360, // Doesn't matter here
                    endian
                };

                //decode_synth_sample_file(input_path, &sys_info)?
                let xma = generate_xma_from_synth_sample(input_path, &sys_info)?;
                let mut file = pikaxe::io::create_new_file(output_path)?;
                file.write_all(&xma)?;
            },
            FileType::Vgs => {
                // Decode (returns interleaved audio samples)
                println!("Decoding...");
                let (sample_data, channels, sample_rate) = decode_vgs_file(input_path)?;

                // Encode to wav
                let encoder = WavEncoder::new(&sample_data, channels, sample_rate);
                encoder.encode_to_file(output_path).map_err(|e| Box::new(e) as Box<dyn Error>)?;
            },
        };

        println!("Wrote output to \"{}\"", output_path.to_str().unwrap_or_default());
        Ok(())
    }
}

fn decode_vgs_file(file_path: &Path) -> Result<(Vec<i16>, u16, u32), Box<dyn Error>> {
    let mut input_file = std::fs::File::open(file_path)?;
    let vgs_file = VgsFile::from_reader(&mut input_file)?;

    let channel_count = vgs_file.get_channel_count();
    let sample_rate = vgs_file.get_sample_rate();

    let channels = vgs_file.decode_samples_as_channels();
    let sample_count = channels.iter().map(|c| c.len()).max().unwrap_or_default();
    let mut interleaved_data = Vec::new();

    // TODO: Figure out more efficient way to do this
    for sample_idx in 0..sample_count {
        for sample in channels.iter().map(|c| c.get(sample_idx).map(|s| *s).unwrap_or_default()) {
            interleaved_data.push(sample);
        }
    }

    Ok((interleaved_data, channel_count as u16, sample_rate))
}

fn decode_str_file(file_path: &Path) -> Result<Vec<i16>, std::io::Error> {
    let mut input_file = std::fs::File::open(file_path)?;

    let mut input_data = Vec::new();
    input_file.read_to_end(&mut input_data)?;

    deinterleave_str(&mut input_data);
    let samples = convert_to_samples(input_data);

    //let mut decoder = ADPCMDecoder::new();
    //let samples = decoder.decode(&input_data);

    Ok(samples)
}

fn generate_xma_from_synth_sample(file_path: &Path, info: &SystemInfo) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut stream = FileStream::from_path_as_read_open(file_path)?;

    // Open synth sample
    let mut synth_sample = SynthSample::default();
    synth_sample.load(&mut stream, info)?;

    // TODO: Return actual error
    if synth_sample.sample_data.encoding != 3 {
        panic!("Unsupported SynthSample with encoding {}", synth_sample.sample_data.encoding);
    }

    // Shouldn't fail...
    Ok(synth_sample.save_as_xma_vec().unwrap())
}

/*fn decode_synth_sample_file(file_path: &Path, info: &SystemInfo) -> Result<(Vec<i16>, u16, u32), Box<dyn Error>> {
    let mut stream = FileStream::from_path_as_read_open(file_path)?;

    // Open synth sample
    let mut synth_sample = SynthSample::default();
    synth_sample.load(&mut stream, info)?;

    // TODO: Return actual error
    if synth_sample.sample_data.encoding != 3 {
        panic!("Unsupported SynthSample with encoding {}", synth_sample.sample_data.encoding);
    }

    // Decode xma
    let samples = decode_xma_packets(
        &synth_sample.sample_data.data,
        synth_sample.sample_data.sample_count
    )?;

    Ok((samples, 1, synth_sample.sample_data.sample_rate as u32))
}*/

fn guess_type_from_magic(file_path: &Path) -> Option<FileType> {
    // Read first 8 bytes of file
    let (magic, version) = {
        let mut magic = [0u8; 4];
        let mut version = [0u8; 4];

        let mut file = std::fs::File::open(file_path).ok()?;
        file.read_exact(&mut magic).ok()?;
        file.read_exact(&mut version).ok()?;

        (magic, version)
    };

    if magic.eq(b"VgS!") {
        return Some(FileType::Vgs);
    }

    let m = i32::from_le_bytes(magic);
    let v = i32::from_le_bytes(version);

    // First interpret as little endian, then try as big endian
    guess_from_versions(m, v, IOEndian::Little)
        .or_else(|| guess_from_versions(m.swap_bytes(), v.swap_bytes(), IOEndian::Big))
}

fn guess_from_versions(m: i32, v: i32, en: IOEndian) -> Option<FileType> {
    match (m, v) {
        (5, 0) => Some(FileType::SynthSample(24, en)),
        (5, 1 | 2) => Some(FileType::SynthSample(25, en)),
        _ => None
    }
}