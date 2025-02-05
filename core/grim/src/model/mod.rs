//mod anim;
//mod draw;
mod export;
mod gltf;
mod import;
//mod group;
//mod mat;
//mod mesh;
mod tex_path;
//mod trans;

#[cfg(feature = "python")] use pyo3::prelude::*;
use std::{error::Error, fs::copy, path::Path};

pub use self::export::*; // TODO: Remove later
pub use self::import::*;
use crate::SystemInfo;
use crate::scene::*;
pub(crate) use self::gltf::*;
pub use self::tex_path::*;

pub(crate) const MILOSPACE_TO_GLSPACE: nalgebra::Matrix4<f32> = nalgebra::Matrix4::new(
    -1.0,  0.0,  0.0, 0.0,
    0.0,  0.0,  1.0, 0.0,
    0.0,  1.0,  0.0, 0.0,
    0.0,  0.0,  0.0, 1.0,
);

pub struct AssetManagager {
    info: SystemInfo,
    groups: Vec<GroupObject>,
    meshes: Vec<MeshObject>,
    materials: Vec<MatObject>,
    textures: Vec<TexPath>,
    trans_anims: Vec<TransAnim>,
}

impl AssetManagager {
    pub fn new(info: SystemInfo) -> AssetManagager {
        AssetManagager {
            info,
            groups: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            textures: Vec::new(),
            trans_anims: Vec::new(),
        }
    }

    pub fn get_group(&self, name: &str) -> Option<&GroupObject> {
        self.groups.iter().find(|g| g.name.eq(name))
    }

    pub fn get_mesh(&self, name: &str) -> Option<&MeshObject> {
        self.meshes.iter().find(|m| m.name.eq(name))
    }

    pub fn get_material(&self, name: &str) -> Option<&MatObject> {
        self.materials.iter().find(|m| m.name.eq(name))
    }

    pub fn get_texture(&self, name: &str) -> Option<&TexPath> {
        self.textures.iter().find(|t| t.name.eq(name))
    }

    pub fn get_trans_anim(&self, name: &str) -> Option<&TransAnim> {
        self.trans_anims.iter().find(|t| t.name.eq(name))
    }

    pub fn add_group(&mut self, group: GroupObject) {
        self.groups.push(group);
    }

    pub fn add_mesh(&mut self, mesh: MeshObject) {
        self.meshes.push(mesh);
    }

    pub fn add_material(&mut self, mat: MatObject) {
        self.materials.push(mat);
    }

    pub fn add_tex(&mut self, tex: TexPath) {
        self.textures.push(tex);
    }

    pub fn add_trans_anim(&mut self, trans_anim: TransAnim) {
        self.trans_anims.push(trans_anim);
    }

    pub fn dump_to_directory<T>(&self, out_dir: T) -> Result<(), Box<dyn Error>> where T: AsRef<Path> {
        // Create output dir
        create_dir_if_not_exists(&out_dir)?;

        for grp in self.groups.iter() {
            // Iterate meshes
            let meshes: Vec<&MeshObject> = (&grp.objects).iter().filter_map(|m| self.get_mesh(m)).collect();
            for mesh in meshes {
                // Write mat
                let mat = self.get_material(&mesh.mat).unwrap();
                let mat_path = out_dir.as_ref().join(&mat.name);
                save_to_file(mat, &mat_path, &self.info)?;
                println!("Wrote {}", &mat.name);

                // Write diffuse tex
                if let Some(tex) = self.get_texture(&mat.diffuse_tex) {
                    let png_path = &tex.png_path;
                    let png_name = png_path.file_name().unwrap().to_str().unwrap().to_ascii_lowercase();
                    let dest_png_path = out_dir.as_ref().join(&png_name);

                    copy(png_path, &dest_png_path)?;
                    println!("Wrote {}", &png_name);
                }

                // Write mesh
                let mesh_path = out_dir.as_ref().join(&mesh.name);
                save_to_file(mesh, &mesh_path, &self.info)?;
                println!("Wrote {}", &mesh.name);
            }

            // Write group
            let group_path = out_dir.as_ref().join(&grp.name);
            save_to_file(grp, &group_path, &self.info)?;
            println!("Wrote {}", &grp.name);
        }

        // Iterate anims
        for trans_anim in self.trans_anims.iter() {
            // Write trans anim
            let trans_anim_path = out_dir.as_ref().join(&trans_anim.name);
            save_to_file(trans_anim, &trans_anim_path, &self.info)?;
            println!("Wrote {}", &trans_anim.name);
        }

        Ok(())
    }
}

pub(crate) fn create_dir_if_not_exists<T>(dir_path: T) -> Result<(), Box<dyn Error>> where T: AsRef<Path> {
    let dir_path = dir_path.as_ref();

    if !dir_path.exists() {
        // Not found, create directory
        std::fs::create_dir_all(&dir_path)?;
    }

    Ok(())
}

pub fn open_model<T>(model_path: T, info: SystemInfo) -> Result<AssetManagager, Box<dyn Error>> where T: AsRef<Path> {
    // Check if path exists first
    verify_path_exists(&model_path, Some("model_path"))?;

    let mut gltf_importer = GLTFImporter::new(&model_path)?;
    gltf_importer.process(info)
}

pub(crate) fn verify_path_exists<T: AsRef<Path>>(path: T, name: Option<&str>) -> Result<(), std::io::Error> {
    path
        .as_ref()
        .try_exists()
        .and_then(|exists| if exists {
            Ok(())
        } else {
            Err(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    if let Some(name) = name {
                        format!("Can't find \"{}\" with path \"{}\"", name, path_as_string(&path))
                    } else {
                        format!("Can't find \"{}\"", path_as_string(&path))
                    }
                )
            )
        })
}

pub(crate) fn path_as_string<'a, T: AsRef<Path>>(path: &'a T) -> &'a str {
    path
        .as_ref()
        .to_str()
        .unwrap()
}

#[pyfunction]
#[cfg(feature = "python")]
pub(crate) fn print_test() -> PyResult<()> {
    Python::with_gil(|_py| {
        println!("Hello python, again! -Ferris");
        Ok(())
    })
}