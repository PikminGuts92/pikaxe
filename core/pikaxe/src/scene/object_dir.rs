use crate::SystemInfo;
use crate::io::{BinaryStream, FileSearchDepth, FileStream, MemoryStream, PathFinder, SeekFrom, Stream};
use crate::scene::*;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use std::error::Error;

use std::path::{Path, PathBuf};

lazy_static! {
    static ref MILO_ENTRY_REGEX: Regex = Regex::new(r"(?i)([/\\][a-z]+[/\\])[^/\\]+$").unwrap();
}

pub enum ObjectDir {
    ObjectDir(ObjectDirBase)
}

pub struct ObjectDirBase {
    pub entries: Vec<Object>,
    pub name: String,
    pub dir_type: String,
    pub sub_dirs: Vec<ObjectDir>,
}

impl ObjectDirBase {
    pub fn new() -> ObjectDirBase {
        ObjectDirBase {
            entries: Vec::new(),
            name: String::new(),
            dir_type: String::new(),
            sub_dirs: Vec::new(),
        }
    }
}

impl<'a> ObjectDir {
    pub fn from_path(path: &Path, _info: &SystemInfo) -> Result<ObjectDir, Box<dyn Error>> {
        let mut obj_dir = ObjectDirBase::new();

        let files = path.find_files_with_depth(FileSearchDepth::Limited(1))?
            .into_iter()
            .filter(|f| MILO_ENTRY_REGEX.is_match(f.to_str().unwrap()))
            .collect::<Vec<PathBuf>>();

        for file_path in files.iter() {
            // Gets file name
            let entry_name = file_path
                .file_name().unwrap()
                .to_str().unwrap()
                .to_owned();

            // Gets directory name as string and converts to pascal casing
            let entry_type = file_path
                .parent().unwrap()
                .file_name().unwrap()
                .to_str().unwrap()
                .chars()
                .enumerate()
                .map(|(i, ch)| match i {
                    0 => ch.to_ascii_uppercase(),
                    _ => ch.to_ascii_lowercase()
                })
                .collect::<String>();

            // Read data from file
            let mut stream = FileStream::from_path_as_read_open(file_path)?;
            let stream_len = stream.len()?;
            let data = stream.read_bytes(stream_len)?;

            // Add entry to collection
            obj_dir.entries.push(Object::Packed(PackedObject {
                name: entry_name,
                object_type: entry_type,
                data
            }));
        }

        Ok(ObjectDir::ObjectDir(obj_dir))
    }

    pub fn get_entries(&'a self) -> &'a Vec<Object> {
        match self {
            ObjectDir::ObjectDir(dir) => &dir.entries
        }
    }

    pub fn get_entries_mut(&'a mut self) -> &'a mut Vec<Object> {
        match self {
            ObjectDir::ObjectDir(dir) => &mut dir.entries
        }
    }

    pub(crate) fn take_entries(&mut self) -> Vec<Object> {
        self.get_entries_mut()
            .drain(..)
            .collect::<Vec<_>>()
    }

    pub fn unpack_entries(&'a mut self, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        #[allow(irrefutable_let_patterns)]
        if let ObjectDir::ObjectDir(obj_dir) = self {
            for entry in obj_dir.entries.iter_mut() {
                if let Some(new_entry) = entry.unpack(info) {
                    *entry = new_entry;
                } else {
                    warn!("Unable to unpack \"{}\" ({})", entry.get_name(), entry.get_type());
                }
            }
        }

        Ok(())
    }
}

impl ObjectDir {
    pub(crate) fn fix_class_name(version: u32, class_name: &mut String) {
        if version >= 25 {
            // Nothing to fix
            return;
        }

        let new_name = match (version, class_name.as_str()) {
            (0..=24, "RenderedTex") => Some("TexRenderer"),
            (0..=24, "CompositeTexture") => Some("LayerDir"),
            (0..=23, "BandFx") => Some("WorldFx"),
            (0..=21, "Slider") => Some("BandSlider"),
            (0..=20, "TextEntry") => Some("BandTextEntry"),
            (0..=19, "Placer") => Some("BandPlacer"),
            (0..=18, "ButtonEx") => Some("BandButton"),
            (0..=18, "LabelEx") => Some("BandLabel"),
            (0..=18, "PictureEx") => Some("BandPicture"),
            (0..=17, "UIPanel") => Some("PanelDir"),
            (0..=15, "WorldInstance") => Some("WorldObject"),
            (0..=14, "View") => Some("Group"),
            (0..=6, "String") => Some("Line"),
            (0..=5, "MeshGenerator") => Some("Generator"),
            (0..=4, "TexMovie") => Some("Movie"),
            _ => None,
        };

        // Update name if needed
        if let Some(name) = new_name {
            *class_name = name.to_owned();
        }
    }
}