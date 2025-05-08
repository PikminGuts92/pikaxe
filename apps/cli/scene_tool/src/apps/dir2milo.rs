use crate::apps::SubApp;
use clap::Parser;

use std::error::Error;

use std::path::Path;


use pikaxe::{Platform, SystemInfo};
use pikaxe::io::*;
use pikaxe::scene::{Object, ObjectDir, PackedObject, Tex};


#[derive(Parser, Debug)]
pub struct Dir2MiloApp {
    #[arg(help = "Path to input directory", required = true)]
    pub dir_path: String,
    #[arg(help = "Path to output milo scene", required = true)]
    pub milo_path: String,
}

// TODO: Get from args
const SYSTEM_INFO: SystemInfo = SystemInfo {
    version: 10,
    platform: Platform::PS2,
    endian: IOEndian::Little,
};

impl SubApp for Dir2MiloApp {
    fn process(&mut self) -> Result<(), Box<dyn Error>> {
        let dir_path = Path::new(&self.dir_path);
        let milo_path = Path::new(&self.milo_path);

        let dir_obj = ObjectDir::from_path(dir_path, &SYSTEM_INFO)?;
        let archive = MiloArchive::from_object_dir(&dir_obj, &SYSTEM_INFO, None)?;

        // Write to file
        let mut stream = FileStream::from_path_as_read_write_create(milo_path)?;
        archive.write_to_stream(&mut stream)?;

        Ok(())
    }
}