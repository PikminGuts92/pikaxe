use crate::apps::{SubApp};
use clap::Parser;

use std::error::Error;

use pikaxe::{Platform, SystemInfo};
use pikaxe::model::*;
use pikaxe::io::*;

#[derive(Parser, Debug)]
pub struct Model2GroupApp {
    #[arg(help = "Path to input model file (.gltf)", required = true)]
    pub model_path: String,
    #[arg(help = "Path to output directory", required = true)]
    pub output_path: String,
}

// TODO: Get from args
const SYSTEM_INFO: SystemInfo = SystemInfo {
    version: 25,
    platform: Platform::PS3,
    endian: IOEndian::Big,
};

impl SubApp for Model2GroupApp {
    fn process(&mut self) -> Result<(), Box<dyn Error>> {
        let asset_man = open_model(&self.model_path, SYSTEM_INFO)?;
        asset_man.dump_to_directory(&self.output_path)
    }
}
