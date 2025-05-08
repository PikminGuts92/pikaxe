use std::error::Error;
use std::path::PathBuf;

use pikaxe::{SystemInfo};
use pikaxe::io::*;
use pikaxe::scene::{ObjectDir};

pub struct MiloLoader {
    pub path: PathBuf,
    pub sys_info: SystemInfo,
    pub obj_dir: ObjectDir,
}

impl MiloLoader {
    pub fn from_path(milo_path: PathBuf) -> Result<Self, Box<dyn Error>> {
        // Open milo
        let mut stream = FileStream::from_path_as_read_open(&milo_path)?;
        let milo = MiloArchive::from_stream(&mut stream)?;

        // Unpack milo
        let system_info = SystemInfo::guess_system_info(&milo, &milo_path);
        let mut obj_dir = milo.unpack_directory(&system_info)?;
        obj_dir.unpack_entries(&system_info)?;

        Ok(Self {
            path: milo_path,
            sys_info: system_info,
            obj_dir
        })
    }
}