use crate::io::{IOEndian, MiloArchive};
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Platform {
    PS2,
    PS3,
    Wii,
    X360,
}

#[derive(Copy, Clone, Debug)]
pub struct SystemInfo {
    pub version: u32,
    pub platform: Platform,
    pub endian: IOEndian, 
}

impl Default for SystemInfo {
    fn default() -> SystemInfo {
        SystemInfo {
            version: 25,
            platform: Platform::X360,
            endian: IOEndian::Big,
        }
    }
}

impl Platform {
    pub fn guess_platform(milo_path: &Path) -> Platform {
        match milo_path.extension() {
            Some(ext) => match ext.to_str() {
                Some("milo_ps2") => Platform::PS2,
                Some("milo_ps3") => Platform::PS3,
                Some("milo_wii") => Platform::Wii,
                Some("milo_xbox") => Platform::X360,
                Some("rnd") => Platform::PS2,
                Some("rnd_ps2") => Platform::PS2,
                Some("gh") => Platform::PS2,
                Some("gz") => Platform::PS2,
                _ => Platform::X360,
            },
            None => Platform::X360,
        }
    }
}

impl SystemInfo {
    pub fn guess_system_info(milo: &MiloArchive, milo_path: &Path) -> SystemInfo {
        let platform = Platform::guess_platform(milo_path);

        // Default: Big endian - RB1
        let mut endian = IOEndian::Big;
        let mut version = 25;

        // Devkit wii is little endian for some reason
        if platform == Platform::X360 || platform == Platform::Wii {
            if let Some((end, ver)) = milo.guess_endian_version() {
                endian = end;
                version = ver;
            }
        } else if platform == Platform::PS2 {
            endian = IOEndian::Little;
            version = milo.get_version(endian).unwrap_or(24); // GH2
        } else {
            version = milo.get_version(endian).unwrap_or(version);
        }

        SystemInfo {
            version,
            platform,
            endian,
        }
    }

    pub fn is_next_gen(&self) -> bool {
        match self.platform {
            Platform::PS3 | Platform::X360 => true,
            _ => false,
        }
    }

    pub fn get_revision(&self) -> u32 {
        match (self.version, self.platform, self.endian) {
            (0..=24, _, _) => 0,
            (25, Platform::X360, IOEndian::Little) => 1,
            _ => 2,
        }
    }
}
