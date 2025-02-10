use crate::apps::SubApp;
use clap::Parser;

use std::error::Error;

use grim::{Platform, SystemInfo};
use grim::model::*;
use grim::io::*;

#[derive(Parser, Debug)]
pub struct AnimApp {
    #[arg(help = "Path to input animation file (.gltf)", required = true)]
    pub anim_path: String,
    #[arg(help = "Path to output directory", required = true)]
    pub output_path: String,
    #[arg(short, long, help = "Use big endian serialization")]
    pub big_endian: bool,
}

impl SubApp for AnimApp {
    fn process(&mut self) -> Result<(), Box<dyn Error>> {
        // TODO: Get from args
        let system_info: SystemInfo = SystemInfo {
            version: 25,
            platform: Platform::X360,
            endian: if self.big_endian {
                IOEndian::Big
            } else {
                IOEndian::Little
            },
        };

        let importer = GltfImporter2::new(&self.anim_path)?;
        let assets = importer.process();

        assets.dump_to_directory(&self.output_path, &system_info)?;

        Ok(())
    }
}
