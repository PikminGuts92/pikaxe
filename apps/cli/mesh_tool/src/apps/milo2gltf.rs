use crate::apps::{SubApp};
use clap::Parser;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use pikaxe::{Platform, SystemInfo};
use pikaxe::model::*;
use pikaxe::io::*;

#[derive(Parser, Debug)]
pub struct Milo2GltfApp {
    #[arg(help = "Path to input milo scene", required = true)]
    pub milo_path: String,
    #[arg(help = "Additional milos")]
    pub extra_milo_paths: Vec<String>,
    #[arg(short = 'o', long, help = "Path to output directory", required = true)]
    pub output_path: String,
    #[arg(short = 'n' , long, help = "Gltf base file name")]
    pub name: Option<String>,
    #[arg(short = 'e' , long, help = "Embed textures as base64")]
    pub embed_textures: bool,
    #[arg(short = 'b' , long, help = "Save as .glb")]
    pub use_glb: bool
}

impl SubApp for Milo2GltfApp {
    fn process(&mut self) -> Result<(), Box<dyn Error>> {
        //let asset_man = open_model(&self.model_path, SYSTEM_INFO)?;
        //asset_man.dump_to_directory(&self.output_path)

        let milo_path = Path::new(&self.milo_path);
        let dir_path = Path::new(&self.output_path);

        if let Some(file_name) = milo_path.file_name() {
            let file_name = file_name.to_str().unwrap_or("file");

            println!("Opening {}", file_name);
        }

        let mut exporter = GltfExporter::with_settings(GltfExportSettings {
            custom_basename: self.name.to_owned(),
            embed_textures: self.embed_textures,
            write_as_binary: self.use_glb,
            output_dir: dir_path.to_path_buf(),
            ..Default::default()
        });
        exporter.add_milo_from_path(milo_path)?;

        // Add extra milos
        for extra_path in self.extra_milo_paths.iter() {
            let extra_path = PathBuf::from(extra_path);
            let extra_path_file_name = extra_path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap();

            println!("Adding {}", extra_path_file_name);
            exporter.add_milo_from_path(extra_path)?;
        }

        exporter.process()?;
        exporter.save_to_fs()

        /*let mut stream: Box<dyn Stream> = Box::new(FileStream::from_path_as_read_open(milo_path)?);
        let milo = MiloArchive::from_stream(&mut stream)?;

        // Guess system info
        let system_info = SystemInfo::guess_system_info(&milo, &milo_path);
        let mut obj_dir = milo.unpack_directory(&system_info)?;
        obj_dir.unpack_entries(&system_info)?;

        let bones = find_bones(&obj_dir);

        let bone_count = calc_total_bone_count(&bones);
        println!("Found {} bones\n", bone_count);

        //print_bones(&bones, 0);

        export_object_dir_to_gltf(&obj_dir, &dir_path, &system_info);

        Ok(())*/
    }
}

fn _print_bones(bones: &Vec<BoneNode>, depth: usize) {
    for bone in bones {
        println!("{}{}", "  ".repeat(depth), bone.object.get_name());

        if !bone.children.is_empty() {
            _print_bones(&bone.children, depth + 1);
        }
    }
}

fn _calc_total_bone_count(bones: &Vec<BoneNode>) -> usize {
    bones
        .iter()
        .map(|b| _calc_total_bone_count(&b.children))
        .sum::<usize>() + bones.len()
}