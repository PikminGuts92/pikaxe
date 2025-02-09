use crate::apps::{GameOptions, SubApp};
use clap::Parser;
use std::cmp::Ordering;
use std::error::Error;
use std::{arch, fs};
use std::path::{Path, PathBuf};
use thiserror::Error;

use grim::{Platform, SystemInfo};
use grim::io::*;
use grim::scene::{EncodedSamples, Object, ObjectDir, ObjectReadWrite, PackedObject, Tex};
use grim::texture::{Bitmap, Image, swap_image_bytes, write_rgba_to_file};


#[derive(Parser, Debug)]
pub struct SaveMiloApp {
    #[arg(help = "Path to input milo scene", required = true)]
    pub in_milo_path: String,
    #[arg(help = "Path to output milo scene", required = true)]
    pub out_milo_path: String,
    #[arg(short = 'm', long, help = "Milo archive version (10, 24, 25)")]
    pub milo_version: Option<u32>,
    #[arg(short = 'b' , long, help = "Use big endian serialization")]
    pub big_endian: Option<bool>,
    #[arg(short = 'u' , long, help = "Leave output milo archive uncompressed")]
    pub uncompressed: bool,
}

impl SubApp for SaveMiloApp {
    fn process(&mut self) -> Result<(), Box<dyn Error>> {
        let in_milo_path = Path::new(&self.in_milo_path);
        let out_milo_path = Path::new(&self.out_milo_path);

        if let Some(file_name) = in_milo_path.file_name() {
            let file_name = file_name.to_str().unwrap();
            println!("Opening {}", file_name);
        }

        // Open milo file
        let mut stream = FileStream::from_path_as_read_open(in_milo_path)?;
        let milo = MiloArchive::from_stream(&mut stream)?;

        // Guess platform info
        let in_sys_info = SystemInfo::guess_system_info(&milo, in_milo_path);
        let mut out_sys_info = SystemInfo {
            platform: Platform::guess_platform(out_milo_path),
            ..in_sys_info
        };

        if in_sys_info.platform.ne(&out_sys_info.platform) && out_sys_info.platform.eq(&Platform::Wii) {
            todo!("Converting milo to wii is not currently supported!");
        }

        if out_sys_info.platform.eq(&Platform::PS3) {
            // Force big endian if ps3
            out_sys_info.endian = IOEndian::Big;
        }

        // Set milo version if provided
        if let Some(mv) = self.milo_version {
            out_sys_info.version = mv;
            println!("Using milo version: {}", mv);
        }

        // Set endian if provided
        if let Some(end) = self.big_endian {
            out_sys_info.endian = match end {
                true => IOEndian::Big,
                _ => IOEndian::Little
            };
            println!("Using endian: {:?}", out_sys_info.endian);

            if in_sys_info.endian.ne(&out_sys_info.endian) {
                println!("WARN: Output endian doesn't match input");
            }
        }

        // Unpack milo
        let mut obj_dir = milo.unpack_directory(&in_sys_info)?;
        unpack_entries(&mut obj_dir, &in_sys_info, false);

        if in_sys_info.platform.ne(&out_sys_info.platform) {
            println!("Converting platform from {:?} to {:?}", in_sys_info.platform, out_sys_info.platform);

            convert_textures(&mut obj_dir, &in_sys_info, &out_sys_info);
        }

        // Uncompress animations
        for obj in obj_dir.get_entries_mut() {
            if let Object::CharClipSamples(ccs) = obj {
                println!("Name: {}", &ccs.name);

                for cbs in [&mut ccs.one, &mut ccs.full] {
                    match &cbs.samples {
                        EncodedSamples::Compressed(bones, samples) => {
                            println!("\tStart: {} bones, {} samples", bones.len(), samples.len());
                        },
                        _ => {}
                    }

                    let samples = cbs.decode_samples(&in_sys_info);
                    println!("\tDecoded: {} samples", samples.len());

                    cbs.samples = EncodedSamples::Uncompressed(samples);

                    cbs.generate_bones_from_samples();
                    cbs.recompute_sizes();
                }
            }
        }

        if in_sys_info.version.ne(&out_sys_info.version) {
            println!("Converting milo version from {:?} to {:?}", in_sys_info.version, out_sys_info.version);
        }

        // Write to new milo archive
        let block_type = self.uncompressed.then(|| BlockType::TypeA);
        let archive = MiloArchive::from_object_dir(&obj_dir, &out_sys_info, block_type)?;

        let mut stream = FileStream::from_path_as_read_write_create(out_milo_path)?;
        archive.write_to_stream(&mut stream)?;

        if let Some(file_name) = out_milo_path.file_name() {
            let file_name = file_name.to_str().unwrap();
            println!("Successfully wrote {}", file_name);
        }

        Ok(())
    }
}

fn unpack_entries(milo_dir: &mut ObjectDir, info: &SystemInfo, all: bool) {
    if all {
        milo_dir.unpack_entries(info).unwrap();
        return;
    }

    let supported_types = [
        "CharClipSamples",
        "CubeTex",
        "Group",
        "Mat",
        "Mesh",
        "Tex",
        "Trans"
    ];

    for entry in milo_dir.get_entries_mut() {
        if !supported_types.contains(&entry.get_type()) {
            continue
        }

        if let Some(new_entry) = entry.unpack(info) {
            *entry = new_entry;
        }
    }
}

fn convert_textures(milo_dir: &mut ObjectDir, in_sys_info: &SystemInfo, out_sys_info: &SystemInfo) {
    for obj in milo_dir.get_entries_mut() {
        if let Object::Tex(tex) = obj {
            transcode_texture(tex, in_sys_info, out_sys_info);
        }
    }
}

fn transcode_texture(tex: &mut Tex, in_sys_info: &SystemInfo, out_sys_info: &SystemInfo) {
    if let Some(bitmap) = &mut tex.bitmap {
        match (bitmap.encoding, &in_sys_info.platform, &out_sys_info.platform) {
            (32, &Platform::X360, _) => {},
            (_, &Platform::PS3, &Platform::X360) | (_, &Platform::X360, &Platform::PS3) => {
                // Just swap bytes to save time
                if !bitmap.raw_data.is_empty() {
                    swap_image_bytes(&mut bitmap.raw_data);
                    println!("Swapped bytes for \"{}\"", tex.name.as_str());
                }
                return;
            },
            _ => {}
        }

        // Decode proper
        if let Some(rgba) = bitmap.unpack_rgba(in_sys_info).ok() {
            println!("Successfully decoded \"{}\"", tex.name.as_str());

            *bitmap = Bitmap::from_image(Image::FromRGBA {
                rgba: rgba.as_slice(),
                width: bitmap.width,
                height: bitmap.height,
                mips: 0,
            }, out_sys_info);
        }
    }
}