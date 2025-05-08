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
    SynthSample(u32, IOEndian)
}

#[derive(Parser, Debug)]
pub struct EncoderApp {
    #[arg(help = "Path to input audio (.wav)", required = true)]
    pub input_path: String,
    #[arg(help = "Path to output audio (.vgs)", required = true)]
    pub output_path: String,
    #[arg(short = 's', long, help = "Sample rate (Default: Use sample rate from input audio)")]
    pub sample_rate: Option<u32>,
}

impl SubApp for EncoderApp {
    fn process(&mut self) -> Result<(), Box<dyn Error>> {
        let _input_path = Path::new(&self.input_path);
        let _output_path = Path::new(&self.output_path);

        todo!()
        //Ok(())
    }
}
