use crate::io::*;
use crate::scene::*;
//use grim_traits::scene::*;
use crate::{Platform, SystemInfo};
use grim_traits::scene::Group;
use itertools::*;
use gltf_json as json;
use grim_gltf::*;
use nalgebra as na;
use serde::ser::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

//type TransObject = dyn Trans + MiloObject;

pub struct BoneNode<'a> {
    pub object: &'a dyn Trans,
    pub children: Vec<BoneNode<'a>>
}

fn get_child_nodes<'a>(parent_name: &str, bone_map: &HashMap<&str, &'a dyn Trans>, child_map: &HashMap<&str, Vec<&dyn Trans>>) -> Vec<BoneNode<'a>> {
    let Some(children) = child_map.get(parent_name) else {
        return Vec::new();
    };

    children
        .iter()
        .sorted_by(|a, b| a.get_name().cmp(b.get_name())) // Sort by name
        .map(|c| BoneNode {
            object: *bone_map.get(c.get_name().as_str()).unwrap(),
            children: get_child_nodes(c.get_name().as_str(), bone_map, child_map)
        })
        .collect()
}

pub fn find_bones<'a>(obj_dir: &'a ObjectDir) -> Vec<BoneNode<'a>> {
    let dir_name = match obj_dir {
        ObjectDir::ObjectDir(base) => base.name.as_str(),
    };

    let bones = obj_dir
        .get_entries()
        .iter()
        .filter_map(|o| match o {
            Object::Mesh(m) if m.faces.is_empty() // GH1 bones
                => Some(m as &dyn Trans),
            Object::Trans(t) => Some(t as &dyn Trans),
            _ => None
        })
        .map(|b| (b.get_name().as_str(), b))
        .collect::<HashMap<_, _>>();

    // Map parent to children
    let child_map = bones
        .iter()
        .fold(HashMap::new(), |mut acc: HashMap<&str, Vec<&'a dyn Trans>>, (_, b)| {
            if b.get_parent().eq(b.get_name()) {
                // If bone references self, ignore
                return acc;
            }

            acc
                .entry(b.get_parent().as_str())
                .and_modify(|e| e.push(*b))
                .or_insert(vec![*b]);

            acc
        });

    let mut root_nodes = Vec::new();

    // Add bones that belong to object dir
    let mut dir_nodes = get_child_nodes(dir_name, &bones, &child_map);
    root_nodes.append(&mut dir_nodes);

    // TODO: Add unparented bones

    root_nodes
}

fn map_bones_to_nodes(dir_name: &str, bones: &Vec<BoneNode>) -> Vec<gltf_json::Node> {
    let mut nodes = Vec::new();

    // Add root obj dir node
    // Ugh... no default derive...
    nodes.push(gltf_json::Node {
        camera: None,
        children: None,
        extensions: None,
        extras: Default::default(),
        matrix: Some([
            -1.0,  0.0,  0.0, 0.0,
            0.0,  0.0,  1.0, 0.0,
            0.0,  1.0,  0.0, 0.0,
            0.0,  0.0,  0.0, 1.0,
        ]),
        mesh: None,
        name: Some(dir_name.to_string()),
        rotation: None,
        scale: None,
        translation: None,
        skin: None,
        weights: None,
    });

    let child_indices = populate_child_nodes(&mut nodes, bones);

    if !child_indices.is_empty() {
        nodes[0].children = Some(child_indices);
    }

    //bones
    //    .into_iter()
    //    .enumerate()
    //    .map(|(i, b)|)
    //    .collect()

    nodes
}

fn populate_child_nodes(nodes: &mut Vec<gltf_json::Node>, bones: &Vec<BoneNode>) -> Vec<gltf_json::Index<gltf_json::Node>> {
    let mut indices = Vec::new();

    for bone in bones {
        let child_indices = populate_child_nodes(nodes, &bone.children);

        let m = bone.object.get_local_xfm();
        //let m = Matrix::identity();

        let mat = na::Matrix4::new(
            // Column-major order...
            m.m11, m.m21, m.m31, m.m41,
            m.m12, m.m22, m.m32, m.m42,

            m.m13, m.m23, m.m33, m.m43,
            m.m14, m.m24, m.m34, m.m44,

            /*m.m11, m.m12, m.m13, m.m14,
            m.m21, m.m22, m.m23, m.m24,
            m.m31, m.m32, m.m33, m.m34,
            m.m41, m.m42, m.m43, m.m44*/
        );

        //let scale_mat = na::Matrix4::new_scaling(1.0);

        /*let trans_mat = na::Matrix4::new(
            -1.0,  0.0,  0.0, 0.0,
            0.0,  0.0,  1.0, 0.0,
            0.0,  1.0,  0.0, 0.0,
            0.0,  0.0,  0.0, 1.0,
        );

        let trans_mat = na::Matrix4::new(
            trans_mat[0], trans_mat[4], trans_mat[8], trans_mat[12],
            trans_mat[1], trans_mat[5], trans_mat[9], trans_mat[13],
            trans_mat[2], trans_mat[6], trans_mat[10], trans_mat[14],
            trans_mat[3], trans_mat[7], trans_mat[11], trans_mat[15],
        );

        // TODO: Apply translation...
        let mat = mat * trans_mat;*/

        //let mat = mat * scale_mat;

        //na::Matrix::from

        //let mut gltf_mat 

        let node = gltf_json::Node {
            camera: None,
            children: if !child_indices.is_empty() {
                Some(child_indices)
            } else {
                None
            },
            extensions: None,
            extras: Default::default(),
            matrix: if mat.is_identity(f32::EPSILON) {
                // Don't add identities
                None
            } else {
                mat
                    .as_slice()
                    .try_into()
                    .ok()
            },
            mesh: None,
            name: Some(bone.object.get_name().to_string()),
            rotation: None,
            scale: None,
            translation: None,
            skin: None,
            weights: None,
        };

        nodes.push(node);
        indices.push(gltf_json::Index::new((nodes.len() - 1) as u32));
    }

    indices
}

fn get_textures<'a>(obj_dir: &'a ObjectDir) -> Vec<&Tex> {
    obj_dir
        .get_entries()
        .iter()
        .filter_map(|e| match e {
            // TODO: Support external textures
            Object::Tex(tex) if tex.bitmap.is_some() => Some(tex),
            _ => None
        })
        .collect()
}

pub fn export_object_dir_to_gltf(obj_dir: &ObjectDir, output_path: &Path, sys_info: &SystemInfo) {
    super::create_dir_if_not_exists(output_path).unwrap();

    let dir_name = match obj_dir {
        ObjectDir::ObjectDir(base) => base.name.as_str(),
    };

    let textures = get_textures(&obj_dir);

    let images = textures
        .into_iter()
        .map(|t| json::Image {
            buffer_view: None,
            mime_type: Some(json::image::MimeType(String::from("image/png"))),
            name: Some(t.get_name().to_owned()),
            uri: {
                use base64::{Engine as _, engine::{self, general_purpose}, alphabet};

                // Decode image
                let rgba = t.bitmap
                    .as_ref()
                    .unwrap()
                    .unpack_rgba(sys_info)
                    .unwrap();

                let (width, height) = t.bitmap
                    .as_ref()
                    .map(|b| (b.width as u32, b.height as u32))
                    .unwrap();

                // Convert to png
                let png_data = crate::texture::write_rgba_to_vec(width, height, &rgba).unwrap();

                // Encode to base64
                let mut str_data = String::from("data:image/png;base64,");
                general_purpose::STANDARD.encode_string(&png_data, &mut str_data);

                Some(str_data)
            },
            extensions: None,
            extras: Default::default()
        })
        .collect();

    let bones = find_bones(&obj_dir);
    let nodes = map_bones_to_nodes(dir_name, &bones);

    let joints = nodes
        .iter()
        .enumerate()
        .map(|(i, _)| json::Index::new(i as u32))
        .collect::<Vec<_>>();

    let root = json::Root {
        asset: json::Asset {
            generator: Some(format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))),
            ..Default::default()
        },
        images,
        nodes: map_bones_to_nodes(dir_name, &bones),
        scene: Some(json::Index::new(0)),
        scenes: vec![
            json::Scene {
                extensions: None,
                extras: Default::default(),
                name: None,
                nodes: vec![json::Index::new(0)],
            }
        ],
        skins: vec![
            json::Skin {
                extensions: None,
                extras: Default::default(),
                inverse_bind_matrices: None,
                joints: joints,
                name: None,
                skeleton: Some(json::Index::new(0))
            }
        ],
        ..Default::default()
    };

    // Write gltf json
    let writer = std::fs::File::create(output_path.join(format!("{dir_name}.gltf"))).expect("I/O error");
    json::serialize::to_writer_pretty(writer, &root).expect("Serialization error");

    // Write gltf buffer
}

#[derive(Default)]
pub struct GltfExportSettings {
    pub custom_basename: Option<String>,
    pub embed_textures: bool,
    pub write_as_binary: bool,
    pub output_dir: PathBuf
}

pub struct ObjectDirData {
    dir: ObjectDir,
    entries: Vec<Object>,
    path: PathBuf,
    info: SystemInfo
}

struct MappedObject<T: MiloObject> {
    parent: Rc<ObjectDirData>,
    object: T
}

impl<T: MiloObject> MappedObject<T> {
    fn new(object: T, parent: Rc<ObjectDirData>) -> MappedObject<T> {
        MappedObject {
            object,
            parent
        }
    }
}

fn is_mesh_joint(m: &MappedObject<MeshObject>) -> bool {
    m.parent.info.version <= 10 && m.object.faces.is_empty()
}

#[derive(Default)]
pub struct GltfExporter {
    object_dirs: Vec<ObjectDirData>, // TODO: Replace with new milo environment?
    dirs_rc: Vec<Rc<ObjectDirData>>,
    settings: GltfExportSettings,
    char_clip_samples: HashMap<String, MappedObject<CharClipSamples>>,
    groups: HashMap<String, MappedObject<GroupObject>>,
    materials: HashMap<String, MappedObject<MatObject>>,
    meshes: HashMap<String, MappedObject<MeshObject>>,
    transforms: HashMap<String, MappedObject<TransObject>>,
    trans_anims: HashMap<String, MappedObject<TransAnim>>,
    textures: HashMap<String, MappedObject<Tex>>,

    // TODO: Move to nested struct?
    gltf: json::Root,
    image_indices: HashMap<String, usize>,
}

impl GltfExporter {
    pub fn new() -> GltfExporter {
        GltfExporter::default()
    }

    pub fn with_settings(settings: GltfExportSettings) -> GltfExporter {
        GltfExporter {
            settings,
            ..Default::default()
        }
    }

    pub fn add_milo_from_path<T: Into<PathBuf>>(&mut self, path: T) -> Result<(), Box<dyn Error>> {
        // TODO: Return custom error type
        let milo_path: PathBuf = path.into();

        // Open milo
        let mut stream = FileStream::from_path_as_read_open(&milo_path)?;
        let milo = MiloArchive::from_stream(&mut stream)?;

        // Guess system info and unpack dir + entries
        let system_info = SystemInfo::guess_system_info(&milo, &milo_path);
        let mut obj_dir = milo.unpack_directory(&system_info)?;
        obj_dir.unpack_entries(&system_info)?;

        let entries = obj_dir.take_entries();

        // Add to list
        self.object_dirs.push(ObjectDirData {
            dir: obj_dir,
            entries: entries,
            path: milo_path,
            info: system_info
        });

        // If basename not set, use milo basename
        if self.settings.custom_basename.is_none() {
            let basename = self.get_basename().to_owned();
            self.settings.custom_basename = Some(basename);
        }

        Ok(())
    }

    fn get_basename(&self) -> &str {
        if let Some(basename) = self.settings.custom_basename.as_ref() {
            // Return custom basename if set
            basename.as_str()
        } else {
            // Use basename from first milo file path
            // Note: Call before mapping objects because this list gets drained... (super hacky)
            self.object_dirs
                .iter()
                .find_map(|dir| dir.path
                    .as_path()
                    .file_stem()
                    .and_then(|fs| fs.to_str()))
                .unwrap_or("output")
        }
    }

    fn map_objects(&mut self) {
        self.char_clip_samples.clear();
        self.groups.clear();
        self.materials.clear();
        self.meshes.clear();
        self.transforms.clear();
        self.textures.clear();

        self.dirs_rc.clear();

        for (_i, mut dir_entry) in self.object_dirs.drain(..).enumerate() {
            let entries = dir_entry.entries.drain(..).collect::<Vec<_>>();
            let parent = Rc::new(dir_entry);

            // Ignore groups + meshes in lower lods or shadow
            // TODO: Work out better way to filter
            let ignored_objects = entries
                .iter()
                .filter_map(|e| match (e, e.get_name()) {
                    (Object::Group(grp), n) if ["shadow", "lod1", "lod2", "lod01", "lod02", "LOD01", "LOD02"]
                        .iter().any(|f| n.contains(f)) => {
                        Some(grp.objects.to_owned().into_iter().chain([n.to_owned()]))
                    },
                    _ => None
                })
                .flatten()
                .collect::<HashSet<_>>();

            for entry in entries {
                if ignored_objects.contains(entry.get_name()) {
                    continue;
                }

                let name = entry.get_name().to_owned();

                match entry {
                    Object::CharClipSamples(ccs) => {
                        self.char_clip_samples.insert(
                            name,
                            MappedObject::new(ccs, parent.clone())
                        );
                    },
                    Object::Group(group) => {
                        self.groups.insert(
                            name,
                            MappedObject::new(group, parent.clone())
                        );
                    },
                    Object::Mat(mat) => {
                        self.materials.insert(
                            name,
                            MappedObject::new(mat, parent.clone())
                        );
                    },
                    Object::Mesh(mesh) => {
                        self.meshes.insert(
                            name,
                            MappedObject::new(mesh, parent.clone())
                        );
                    },
                    Object::Tex(tex) => {
                        self.textures.insert(
                            name,
                            MappedObject::new(tex, parent.clone())
                        );
                    },
                    Object::Trans(trans) => {
                        self.transforms.insert(
                            name,
                            MappedObject::new(trans, parent.clone())
                        );
                    },
                    Object::TransAnim(trans_anim) => {
                        self.trans_anims.insert(
                            name,
                            MappedObject::new(trans_anim, parent.clone())
                        );
                    },
                    _ => {}
                }
            }

            self.dirs_rc.push(parent);
        }

        // Hacky way to default parent to fix skeleton
        // Find first parent containing bones
        let parent_skeleton = self.transforms
            .values()
            .map(|t| (&t.parent, true))
            .chain(self.meshes.values().map(|m| (&m.parent, is_mesh_joint(m))))
            .find_map(|t| match t {
                (parent, true) => Some(parent.clone()),
                _ => None
            });

        let Some(parent_skeleton) = parent_skeleton else {
            return;
        };

        // Update meshes with bones to reference parent skeleton
        /*self.meshes
            .values_mut()
            .filter(|m| match m {
                (parent, _, true) if m.as_ref().path.ne(&parent_skeleton.as_ref().path) => {
                    todo!()
                },
                _ => todo!()
            });*/

        let mut new_children = HashSet::new();

        for (_, m) in self.meshes.iter_mut() {
            // Ignore meshes in skeleton dir
            if m.parent.as_ref().path.eq(&parent_skeleton.as_ref().path) {
                 continue;
            }

            if m.object.bones.iter().any(|b| !b.name.is_empty()) {
                // Update mesh parent if at least one bone found
                /*m.object.parent = match &parent_skeleton.dir {
                    ObjectDir::ObjectDir(base) => base.name.to_owned(),
                };*/

                let dir_name = match &m.parent.dir {
                    ObjectDir::ObjectDir(base) => &base.name,
                };

                if !new_children.contains(dir_name) {
                    new_children.insert(dir_name.to_owned());
                }
            }
        }

        // Add empty trans objects to link children to parent skeleton
        for child_name in new_children.drain() {
            self.transforms.insert(child_name.to_owned(), MappedObject {
                parent: parent_skeleton.clone(),
                object: TransObject {
                    name: child_name,
                    parent: match &parent_skeleton.dir {
                        ObjectDir::ObjectDir(base) => base.name.to_owned(),
                    },
                    ..Default::default()
                }
            });
        }

        /*let parent_skeleton = self.transforms
            .values()
            .map(|t| (&t.parent, &t.object as &dyn Trans, true))
            .chain(self.meshes.values().map(|m| (&m.parent, &m.object as &dyn Trans, is_mesh_joint(m))))
            .filter(|t| match t {
                (parent, _, true) if parent.as_ref().path.ne(&self.dirs_rc[0].as_ref().path) => {
                    todo!()
                },
                _ => todo!()
            });*/
    }

    fn get_transform<'a>(&'a self, name: &str) -> Option<&'a dyn Trans> {
        self.transforms
            .get(name)
            .map(|t| &t.object as &dyn Trans)
            .or(self.groups.get(name).map(|g| &g.object as &dyn Trans))
            .or(self.meshes.get(name).map(|m| &m.object as &dyn Trans))
    }

    fn get_mesh<'a>(&'a self, name: &str) -> Option<&MeshObject> {
        self.meshes
            .get(name)
            .map(|m| &m.object)
    }

    fn process_node<'a>(&'a self, gltf: &mut json::Root, name: &'a str, child_map: &HashMap<&'a str, Vec<&'a str>>, depth: usize) -> usize {
        let node_index = gltf.nodes.len();

        // Get + compute transform matrix
        let node_matrix = match (self.get_transform(name), depth) {
            (Some(trans), 0) => {
                let m = trans.get_world_xfm();

                let mat = na::Matrix4::new(
                    // Column-major order...
                    m.m11, m.m21, m.m31, m.m41,
                    m.m12, m.m22, m.m32, m.m42,
                    m.m13, m.m23, m.m33, m.m43,
                    m.m14, m.m24, m.m34, m.m44
                );

                mat * super::MILOSPACE_TO_GLSPACE
            },
            (Some(trans), _) => {
                let m = trans.get_local_xfm();

                na::Matrix4::new(
                    // Column-major order...
                    m.m11, m.m21, m.m31, m.m41,
                    m.m12, m.m22, m.m32, m.m42,
                    m.m13, m.m23, m.m33, m.m43,
                    m.m14, m.m24, m.m34, m.m44
                )
            },
            (None, 0) => super::MILOSPACE_TO_GLSPACE,
            _ => na::Matrix4::identity()
        };

        // Deconstruct into individual parts
        let (translate, rotation, scale) = decompose_trs(node_matrix);

        gltf.nodes.push(gltf_json::Node {
            camera: None,
            children: None,
            extensions: None,
            extras: Default::default(),
            matrix: None,
            mesh: None,
            name: Some(name.to_owned()),
            // Don't add identities
            rotation: if rotation.eq(&na::UnitQuaternion::identity()) {
                None
            } else {
                Some(json::scene::UnitQuaternion([
                    rotation[0],
                    rotation[1],
                    rotation[2],
                    rotation[3]
                ]))
            },
            scale: if scale.eq(&na::Vector3::from_element(1.0)) {
                None
            } else {
                Some([
                    scale[0],
                    scale[1],
                    scale[2]
                ])
            },
            translation: if translate.eq(&na::Vector3::zeros()) {
                None
            } else {
                Some([
                    translate[0],
                    translate[1],
                    translate[2]
                ])
            },
            skin: None,
            weights: None,
        });

        if let Some(children) = child_map.get(name) {
            let mut child_indices = Vec::new();

            for child_name in children {
                /*if !self.transforms.contains_key(*child_name) {
                    continue;
                }*/

                let idx = self.process_node(gltf, child_name, child_map, depth + 1);
                child_indices.push(gltf_json::Index::new(idx as u32));
            }

            if !child_indices.is_empty() {
                gltf.nodes[node_index].children = Some(child_indices);
            }
        }

        node_index
    }

    fn load_external_texture(&self, tex: &Tex, system_info: &SystemInfo, milo_path: &Path) -> Option<crate::texture::Bitmap> {
        // TODO: Clean all this crap up
        use std::io::Read;

        println!("{milo_path:?}");

        // Load external texture
        let milo_dir_path = milo_path.parent().unwrap();

        // Insert "gen" sub folder
        // TODO: Support loading from milo gen folder too
        let ext_img_path = match tex.ext_path.rfind('/') {
            Some(_) => todo!("Support external textures in nested relative path"), // Use [..]
            None => milo_dir_path.join("gen").join(&tex.ext_path),
        };

        let ext_img_file_stem = ext_img_path.file_stem().and_then(|fs| fs.to_str()).unwrap();
        let ext_img_path_dir = ext_img_path.parent().unwrap();

        let files = ext_img_path_dir.find_files_with_depth(FileSearchDepth::Immediate).unwrap();

        // TODO: Do case-insensitive compare
        let matching_file = files
            .iter()
            .find(|f| f
                .file_stem()
                //.is_some_and(|fs| fs.eq_ignore_ascii_case(ext_img_file_stem))
                .and_then(|fs| fs.to_str())
                .is_some_and(|fs| fs.starts_with(ext_img_file_stem))
            );

        if let Some(file_path) = matching_file {
            log::info!("Found external texture file!\n\t{file_path:?}");

            let data = if file_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("gz")) {
                // File is gz compressed
                let mut file = std::fs::File::open(file_path).unwrap();

                // Read to buffer
                let mut file_data = Vec::new();
                file.read_to_end(&mut file_data).unwrap();

                // Inflate
                crate::io::inflate_gzip_block_no_buffer(&file_data).unwrap()
            } else {
                let mut file = std::fs::File::open(file_path).unwrap();

                // Read to buffer
                let mut file_data = Vec::new();
                file.read_to_end(&mut file_data).unwrap();

                file_data
            };

            let mut stream = crate::io::MemoryStream::from_slice_as_read(&data);
            let bitmap = crate::texture::Bitmap::from_stream(&mut stream, system_info);

            if bitmap.is_ok() {
                log::info!("Successfully opened bitmap");
            } else {
                log::warn!("Error opening bitmap");
            }

            return bitmap.ok();
        }

        None
    }

    fn process_textures(&self, gltf: &mut json::Root) -> HashMap<String, usize> {
        let mut image_indices = HashMap::new();

        gltf.samplers = vec![
            json::texture::Sampler {
                mag_filter: Some(json::validation::Checked::Valid(json::texture::MagFilter::Linear)),
                min_filter: Some(json::validation::Checked::Valid(json::texture::MinFilter::Nearest)),
                wrap_s: json::validation::Checked::Valid(json::texture::WrappingMode::Repeat),
                wrap_t: json::validation::Checked::Valid(json::texture::WrappingMode::Repeat),
                ..Default::default()
            }
        ];

        (gltf.images, gltf.textures) = self.textures
            .values()
            .sorted_by(|a, b| a.object.get_name().cmp(b.object.get_name()))
            .filter(|t| t.object.bpp != 24) // TODO: Support 24bpp textures...
            .enumerate()
            .map(|(i, mt)| {
                let t = &mt.object;
                let sys_info = &mt.parent.info;
                let milo_path = mt.parent.path.as_path();

                // Remove .tex extension
                // TODO: Use more robust method
                let image_name = t.get_name().replace(".tex", ".png");

                image_indices.insert(t.get_name().to_owned(), i);

                let image = json::Image {
                    buffer_view: None,
                    mime_type: Some(json::image::MimeType(String::from("image/png"))),
                    name: Some(image_name.to_owned()),
                    uri: {
                        use base64::{Engine as _, engine::{self, general_purpose}, alphabet};

                        let mut ext_tex = None;

                        // Decode image
                        let (rgba, (width, height)) = t.bitmap
                            .as_ref()
                            .or_else(|| {
                                // Load external texture
                                ext_tex = self.load_external_texture(t, sys_info, milo_path);
                                ext_tex.as_ref()
                            })
                            .map(|b| (
                                b.unpack_rgba(sys_info).unwrap(),
                                (b.width as u32, b.height as u32)
                            ))
                            .unwrap();

                        /*let (width, height) = t.bitmap
                            .as_ref()
                            .map(|b| (b.width as u32, b.height as u32))
                            .unwrap();*/

                        // Convert to png
                        let png_data = crate::texture::write_rgba_to_vec(width, height, &rgba).unwrap();

                        if self.settings.embed_textures {
                            // Encode to base64
                            let mut str_data = String::from("data:image/png;base64,");
                            general_purpose::STANDARD.encode_string(&png_data, &mut str_data);

                            Some(str_data)
                        } else {
                            // Write as external file
                            let output_dir = self.settings.output_dir.as_path();
                            super::create_dir_if_not_exists(output_dir).unwrap();

                            let png_path = output_dir.join(&image_name);

                            let mut writer = std::fs::File::create(&png_path).unwrap();
                            writer.write_all(&png_data).unwrap();

                            println!("Wrote \"{image_name}\"");

                            Some(image_name)
                        }
                    },
                    extensions: None,
                    extras: Default::default()
                };

                let texture = json::Texture {
                    name: Some(t.get_name().to_owned()),
                    sampler: Some(json::Index::new(0u32)),
                    source: json::Index::new(i as u32), // Image index
                    extensions: None,
                    extras: Default::default()
                };

                (image, texture)
            })
            .fold((Vec::new(), Vec::new()), |(mut imgs, mut texs), (img, tex)| {
                imgs.push(img);
                texs.push(tex);
                (imgs, texs)
            });

        image_indices
    }

    fn process_materials(&self, gltf: &mut json::Root, tex_map: &HashMap<String, usize>) -> HashMap<String, usize> {
        let mut mat_indices = HashMap::new();

        gltf.materials = self.materials
            .values()
            .sorted_by(|a, b| a.object.get_name().cmp(b.object.get_name()))
            .enumerate()
            .map(|(i, mm)| {
                let mat = &mm.object;
                let diff_tex = tex_map.get(&mat.diffuse_tex);
                let _norm_tex = tex_map.get(&mat.normal_map);
                let _spec_tex = tex_map.get(&mat.specular_map);

                mat_indices.insert(mat.get_name().to_owned(), i);

                json::Material {
                    name: Some(mat.get_name().to_owned()),
                    pbr_metallic_roughness: json::material::PbrMetallicRoughness {
                        base_color_texture: diff_tex
                            .map(|d| json::texture::Info {
                                index: json::Index::new(*d as u32),
                                tex_coord: 0,
                                extensions: None,
                                extras: Default::default()
                            }),
                        //base_color_factor:
                        ..Default::default()
                    },
                    emissive_factor: json::material::EmissiveFactor([0.0f32; 3]),
                    alpha_mode: json::validation::Checked::Valid(json::material::AlphaMode::Mask),
                    double_sided: true,
                    ..Default::default()
                }
            })
            .collect();

        mat_indices
    }

    fn find_skins(&self, gltf: &mut json::Root, acc_builder: &mut AccessorBuilder) -> HashMap<String, (usize, usize)> {
        let root_indices = gltf
            .scenes[0]
            .nodes
            .iter()
            .map(|n| n.value())
            //.filter(|_| false)
            .collect::<Vec<_>>();

        let mut skins = Vec::new();
        let mut bone_indices = HashMap::new();

        for (i, idx) in root_indices.into_iter().enumerate() {
            let mut joints = Vec::new();
            self.find_joints(gltf, idx, &mut joints, na::Matrix4::identity(), 0);

            if !joints.is_empty() {
                // TODO: Figure out how to handle when nested
                let root_joint = idx;

                // Sort by index
                joints.sort_by(|(a, _), (b, _)| a.cmp(b));

                for (j, _) in joints.iter() {
                    let node_name = gltf.nodes[*j].name.as_ref().unwrap();
                    bone_indices.insert(node_name.to_owned(), (skins.len(), *j)); // (Skin idx, node idx)
                }

                // Add ibm list to accessors
                let ibm_idx = acc_builder.add_array(
                    format!("skin_{i}"),
                    joints
                        .iter()
                        .map(|(_, m)| m.as_slice().try_into().unwrap_or_default())
                        .collect::<Vec<[f32; 16]>>(),
                    BufferType::Skin
                );

                skins.push(json::Skin {
                    extensions: None,
                    extras: Default::default(),
                    inverse_bind_matrices: ibm_idx
                        .map(|i| json::Index::new(i as u32)),
                    joints: joints
                        .into_iter()
                        .map(|(j, _)| json::Index::new(j as u32))
                        .collect(),
                    name: None,
                    skeleton: Some(json::Index::new(root_joint as u32))
                });
            }
        }

        gltf.skins = skins;
        bone_indices
    }

    fn find_joints(&self, gltf: &json::Root, idx: usize, joints: &mut Vec<(usize, na::Matrix4<f32>)>, parent_mat: na::Matrix4<f32>, depth: usize) {
        let (node_name, children, mat) = gltf
            .nodes
            .get(idx)
            .map(|n| (
                n.name.as_ref().unwrap(),
                &n.children,
                // Matrix on node was already decomposed into individual T*R*S parts
                /*parent_mat * n.matrix
                    .map(|m| na::Matrix4::from_column_slice(&m))
                    .or_else(|| {
                        // Re-construct into trs
                        // TODO: Probably don't deconstruct in first place
                        let trans = n.translation
                            .map(na::Vector3::from)
                            .unwrap_or_else(na::Vector3::zeros);

                        let rotate = n.rotation
                            .map(|json::scene::UnitQuaternion([x, y, z, w])|
                                na::UnitQuaternion::from_quaternion(
                                    na::Quaternion::new(w, x, y, z)
                                )
                            )
                            .unwrap_or_else(na::UnitQuaternion::identity);

                        let scale = n.scale
                            .map(na::Vector3::from)
                            .unwrap_or_else(|| na::Vector3::from_element(1.0));

                        Some((na::Matrix4::identity()
                            .append_translation(&trans) *
                            rotate.to_homogeneous())
                            .append_nonuniform_scaling(&scale))
                    })
                    .unwrap_or_default()*/
                if depth == 0 {
                    na::Matrix4::identity()
                } else {
                    parent_mat * n
                        .name
                        .as_ref()
                        .and_then(|n| self.get_transform(n))
                        .map(|trans| {
                            let m = trans.get_local_xfm();

                            na::Matrix4::new(
                                // Column-major order...
                                m.m11, m.m21, m.m31, m.m41,
                                m.m12, m.m22, m.m32, m.m42,
                                m.m13, m.m23, m.m33, m.m43,
                                m.m14, m.m24, m.m34, m.m44
                            )
                        })
                        .unwrap_or_default()
                }
            ))
            .unwrap();

        // Is a joint if Trans or Mesh w/ no faces
        let is_joint = self.transforms.contains_key(node_name.as_str())
            || self.meshes.get(node_name.as_str()).map(is_mesh_joint).unwrap_or_default();

        if is_joint {
            // Calculate inverse bind matrix (shouldn't fail)
            // Also convert to gl space
            /*let mut ibm = if depth > 0 {
                (mat * super::MILOSPACE_TO_GLSPACE).try_inverse().unwrap_or_default()
            } else {
                mat.try_inverse().unwrap_or_default()
            };*/

            let mut ibm = mat.try_inverse().unwrap_or_default();

            /*if depth == 0 {
                ibm *= super::MILOSPACE_TO_GLSPACE
            }*/

            ibm[15] = 1.0; // Force for precision

            // Add index to joint list
            joints.push((idx, ibm));
        }

        if let Some(children) = children {
            // Traverse children
            for child in children {
                self.find_joints(gltf, child.value(), joints, mat, depth + 1);
            }
        }
    }

    fn process_accessor_data(&self, gltf: &mut json::Root) {
        //let mut acc_indices = HashMap::new();

        /*gltf.accessors = self.meshes
            .values()
            .map(|m| &m.object)
            .filter(|m| !m.faces.is_empty())
            .fold((0, 0, 0, 0), |(vn, uv, wt, fc), m| (
                vn + (m.vertices.len() * 12 * 2), // Verts + norms
                uv + (m.vertices.len() * 8),      // UVs
                wt + (m.vertices.len() * 16 * 2), // Weights + tangents
                fc + (m.faces.len() * 6)          // Faces
            ));*/

        let (bv_verts_norms, bv_uvs, bv_weights_tans, mut bv_faces) = self.meshes
            .values()
            .map(|m| &m.object)
            .filter(|m| !m.faces.is_empty())
            .fold((0, 0, 0, 0), |(vn, uv, wt, fc), m| (
                vn + (m.vertices.len() * 12 * 2), // Verts + norms
                uv + (m.vertices.len() * 8),      // UVs
                wt + (m.vertices.len() * 16 * 2), // Weights + tangents
                fc + (m.faces.len() * 6)          // Faces
            ));

        // Make multiple of 4
        bv_faces = align_to_multiple_of_four(bv_faces);
        let total_size = bv_verts_norms + bv_uvs + bv_weights_tans + bv_faces;

        gltf.buffers = vec![{
            use base64::{Engine as _, engine::{self, general_purpose}, alphabet};

            // TODO: Encode actual data...
            let bin_data = vec![0u8; total_size];

            let mut str_data = String::from("data:application/octet-stream;base64,");
            general_purpose::STANDARD.encode_string(&bin_data, &mut str_data);
            
            json::Buffer {
                name: None,
                byte_length: total_size.into(),
                uri: Some(str_data),
                extensions: None,
                extras: Default::default()
            }
        }];

        gltf.buffer_views = vec![
            json::buffer::View {
                name:  Some(String::from("verts_norms")),
                byte_length: bv_verts_norms.into(),
                byte_offset: Some(0u64.into()),
                byte_stride: Some(json::buffer::Stride(12)),
                buffer: json::Index::new(0),
                target: None,
                extensions: None,
                extras: Default::default()
            },
            json::buffer::View {
                name:  Some(String::from("uvs")),
                byte_length: bv_uvs.into(),
                byte_offset: Some(bv_verts_norms.into()),
                byte_stride: Some(json::buffer::Stride(8)),
                buffer: json::Index::new(0),
                target: None,
                extensions: None,
                extras: Default::default()
            },
            json::buffer::View {
                name:  Some(String::from("weights_tans")),
                byte_length: bv_weights_tans.into(),
                byte_offset: Some((bv_verts_norms + bv_uvs).into()),
                byte_stride: Some(json::buffer::Stride(16)),
                buffer: json::Index::new(0),
                target: None,
                extensions: None,
                extras: Default::default()
            },
            json::buffer::View {
                name:  Some(String::from("faces")),
                byte_length: bv_faces.into(),
                byte_offset: Some((bv_verts_norms + bv_uvs + bv_weights_tans).into()),
                byte_stride: None,
                buffer: json::Index::new(0),
                target: None,
                extensions: None,
                extras: Default::default()
            }
        ];
    }

    fn process_meshes(&self, gltf: &mut json::Root, acc_builder: &mut AccessorBuilder, mat_map: &HashMap<String, usize>) -> HashMap<String, usize> {
        let milo_meshes = self
            .meshes
            .values()
            .filter(|m| !is_mesh_joint(m))
            .map(|m| &m.object)
            .sorted_by(|a, b| a.get_name().cmp(b.get_name()))
            .collect::<Vec<_>>();

        // Get skins
        // Compute relative skin indices
        let local_joint_map = gltf
            .skins
            .iter()
            .map(|s| s.joints
                .iter()
                .enumerate()
                .map(|(ji, jnode)| (
                    // Get local skin index of joint
                    gltf.nodes[jnode.value()].name.as_ref().unwrap(),
                    ji
                ))
                .collect::<Vec<_>>())
            .enumerate()
            .fold(HashMap::new(), |mut acc, (si, mut joints)| {
                joints
                    .drain(..)
                    .for_each(|(name, ji)| {
                        acc.insert(name, (si, ji));
                    });

                acc
            });

        // Map mesh name to node index
        let mesh_node_map = gltf
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| node.name.as_ref().map(|n| (n.to_owned(), i)))
            .collect::<HashMap<_, _>>();

        // Track skinned meshes
        let mut meshes_to_update = Vec::new();

        let mut meshes = Vec::new();
        let mut mesh_map = HashMap::new();

        for mesh in milo_meshes {
            let has_base_texture = mat_map
                .get(&mesh.mat)
                .map(|mi| gltf.materials[*mi].pbr_metallic_roughness.base_color_texture.is_some())
                .unwrap_or_default();

            /*let bone_offsets = mesh
                .bones
                .iter()
                //.filter(|b| false)
                .map(|b| {
                    let m = &b.trans;

                    na::Matrix4::new(
                        // Column-major order...
                        m.m11, m.m21, m.m31, m.m41,
                        m.m12, m.m22, m.m32, m.m42,
    
                        m.m13, m.m23, m.m33, m.m43,
                        m.m14, m.m24, m.m34, m.m44
                    )
                })
                .collect::<Vec<_>>();

            let translated_pos = mesh
                .get_vertices()
                .iter()
                //.map(|v| [v.pos.x, v.pos.y, v.pos.z])
                .map(|v| {
                    let pos = na::Vector3::new(v.pos.x, v.pos.y, v.pos.z);

                    // Calculate weighted offsets from bones
                    let off = v.bones
                        .iter()
                        .zip(&v.weights)
                        .filter_map(|(b, w)| bone_offsets
                            .get(*b as usize)
                            .map(|bo| bo.transform_vector(&na::Vector3::from_element(0.0)).scale(*w)))
                        .sum::<na::Vector3<f32>>();

                    // Add offset to pos
                    let t = pos + off;
                    [t[0], t[1], t[2]]
                });*/

            let pos_idx = acc_builder.add_array(
                format!("{}_pos", mesh.get_name()),
                mesh.get_vertices().iter().map(|v| [v.pos.x, v.pos.y, v.pos.z]),
                BufferType::Mesh
            );

            let norm_idx = acc_builder.add_array(
                format!("{}_norm", mesh.get_name()),
                mesh.get_vertices().iter().map(|v| {
                    // PS2 norms aren't normalized?
                    let v = na::Vector3::new(v.pos.x, v.pos.y, v.pos.y).normalize();
                    [v[0], v[1], v[2]]
                }),
                BufferType::Mesh
            );

            // Don't add uvs if no texture associated
            let uv_idx = if has_base_texture {
                acc_builder.add_array(
                    format!("{}_uv", mesh.get_name()),
                    mesh.get_vertices().iter().map(|v| [v.uv.u, v.uv.v]),
                    BufferType::Mesh
                )
            } else {
                None
            };

            let mut weight_idx = None;
            let mut bone_idx = None;

            // Get joint info
            // Convert local bone offset to skin joint offset
            let joint_translate_map = mesh
                .bones
                .iter()
                .enumerate()
                .flat_map(|(i, b)| local_joint_map
                    .get(&b.name)
                    .map(|j| (i, *j)))
                .collect::<HashMap<_, _>>();

            // Only add if bones found
            if !joint_translate_map.is_empty() {
                // Convert mesh bones to vert bones
                let bones = [
                    joint_translate_map.get(&0).map(|(_, b)| *b as u16).unwrap_or_default(),
                    joint_translate_map.get(&1).map(|(_, b)| *b as u16).unwrap_or_default(),
                    joint_translate_map.get(&2).map(|(_, b)| *b as u16).unwrap_or_default(),
                    joint_translate_map.get(&3).map(|(_, b)| *b as u16).unwrap_or_default(),
                ];

                // Create combined bones + weights
                let (conv_weights, conv_bones) = mesh.get_vertices()
                    .iter()
                    .map(|v| {
                        let w = v.weights;
                        let mut b = bones.to_owned();

                        // If weight is 0.0, set bone index to 0
                        for (b, w) in b.iter_mut().zip_eq(w) {
                            if w.eq(&0.0) {
                                *b = 0;
                            }
                        }

                        (w, b)
                    })
                    .fold((Vec::new(), Vec::new()), |(mut ws, mut bs), (w, b)| {
                        ws.push(w);
                        bs.push(b);

                        (ws, bs)
                    });

                // Add bone weights
                weight_idx = acc_builder.add_array(
                    format!("{}_weight", mesh.get_name()),
                    conv_weights,
                    BufferType::Mesh
                );

                // Add bone indices
                bone_idx = acc_builder.add_array(
                    format!("{}_bone", mesh.get_name()),
                    conv_bones,
                    BufferType::Mesh
                );

                // Get first skin (all bones should use the same skin...)
                // Still need to check in case bone isn't found
                let skin_idx = (0..4).find_map(|i| joint_translate_map.get(&i).map(|(s, _)| *s));

                if let Some(skin_idx) = skin_idx {
                    let node_idx = mesh_node_map.get(mesh.get_name());

                    if let Some(node_idx) = node_idx {
                        meshes_to_update.push((*node_idx, skin_idx));
                    }
                }
            }

            // Ignore tangents for now
            let tan_idx: Option<usize> = None;
            /*let tan_idx = acc_builder.add_array(
                format!("{}_tan", mesh.get_name()),
                mesh.get_vertices().iter().map(|v| [v.tangent.x, v.tangent.y, v.tangent.z, v.tangent.w])
            );*/

            // Need to be scalar for some reason
            let face_idx = acc_builder.add_scalar(
                format!("{}_face", mesh.get_name()),
                mesh.get_faces().iter().map(|f| f.to_owned()).flatten(),
                BufferType::Mesh
            );

            let mesh_idx = meshes.len();

            meshes.push(json::Mesh {
                name: Some(mesh.get_name().to_owned()),
                primitives: vec![
                    json::mesh::Primitive {
                        attributes: {
                            let mut map = BTreeMap::new();

                            // Add positions
                            if let Some(acc_idx) = pos_idx {
                                map.insert(
                                    json::validation::Checked::Valid(json::mesh::Semantic::Positions),
                                    json::Index::new(acc_idx as u32)
                                );
                            }

                            // Add normals
                            if let Some(acc_idx) = norm_idx {
                                map.insert(
                                    json::validation::Checked::Valid(json::mesh::Semantic::Normals),
                                    json::Index::new(acc_idx as u32)
                                );
                            }

                            // Add uvs
                            if let Some(acc_idx) = uv_idx {
                                map.insert(
                                    json::validation::Checked::Valid(json::mesh::Semantic::TexCoords(0)),
                                    json::Index::new(acc_idx as u32)
                                );
                            }

                            // Add weights
                            if let Some(acc_idx) = weight_idx {
                                map.insert(
                                    json::validation::Checked::Valid(json::mesh::Semantic::Weights(0)),
                                    json::Index::new(acc_idx as u32)
                                );
                            }

                            // Add bones
                            if let Some(acc_idx) = bone_idx {
                                map.insert(
                                    json::validation::Checked::Valid(json::mesh::Semantic::Joints(0)),
                                    json::Index::new(acc_idx as u32)
                                );
                            }

                            // Add tangents
                            if let Some(acc_idx) = tan_idx {
                                map.insert(
                                    json::validation::Checked::Valid(json::mesh::Semantic::Tangents),
                                    json::Index::new(acc_idx as u32)
                                );
                            }

                            map
                        },
                        indices: face_idx
                            .map(|idx| json::Index::new(idx as u32)),
                        material: mat_map
                            .get(&mesh.mat)
                            .map(|idx| json::Index::new(*idx as u32)),
                        mode: json::validation::Checked::Valid(gltf::mesh::Mode::Triangles),
                        targets: None,
                        extras: Default::default(),
                        extensions: None
                    },
                ],
                weights: None,
                extras: Default::default(),
                extensions: None
            });

            // Update map
            mesh_map.insert(mesh.get_name().to_owned(), mesh_idx);
        }

        // Update skins for each mesh node updated
        for (node_idx, skin_idx) in meshes_to_update {
            gltf.nodes[node_idx].skin = Some(json::Index::new(skin_idx as u32));
            //gltf.nodes[node_idx].translation = None;
            //gltf.nodes[node_idx].rotation = None;
            //gltf.nodes[node_idx].scale = None;
        }

        // Assign meshes and return mesh indices
        gltf.meshes = meshes;
        mesh_map
    }

    fn final_process_nodes(&self, gltf: &mut json::Root, mesh_map: &HashMap<String, usize>, joint_map: &HashMap<String, (usize, usize)>, acc_builder: &mut AccessorBuilder) {
        // Useless code... does nothing
        for i in 0..gltf.nodes.len() {
            // Get node name
            let Some(node_name) = gltf.nodes[i].name.as_ref().map(|n| n.to_owned()) else {
                continue;
            };

            if let Some(mesh_idx) = mesh_map.get(&node_name) {
                // Update mesh index for node
                gltf.nodes[i].mesh = Some(json::Index::new(*mesh_idx as u32));
            } else {
                // Can't add skin without mesh
                continue;
            }

            if let Some((skin_idx, _)) = joint_map.get(&node_name) {
                // Update skin index for node
                gltf.nodes[i].skin = Some(json::Index::new(*skin_idx as u32));
                //gltf.nodes[i].translation = None;
                //gltf.nodes[i].rotation = None;
                //gltf.nodes[i].scale = None;
            }

            /*if gltf.nodes[i].mesh.is_some() {
                gltf.nodes[i].translation = None;
                gltf.nodes[i].rotation = None;
                gltf.nodes[i].scale = None;
            }*/
        }

        for i in 0..gltf.skins.len() {
            // Move skinned mesh nodes to root skeleton node
            let old_skeleton_node_idx = gltf
                .skins[i]
                .skeleton
                .map(|s| s.value())
                .unwrap();

            let scene_idx = 0; // TODO: Get from skin somehow...?

            self.move_skinned_mesh_to_skeleton_root(gltf, scene_idx, old_skeleton_node_idx, old_skeleton_node_idx, acc_builder);
        }

        /*let milo_meshes = self
            .meshes
            .values()
            .filter(|m| !is_mesh_joint(m))
            .map(|m| m.object.get_name())
            .collect::<HashSet<_>>();

        let skin_nodes = gltf
            .skins
            .iter()
            .map(|s| s.skeleton.unwrap().value());*/
    }

    fn move_skinned_mesh_to_skeleton_root(&self, gltf: &mut json::Root, scene_idx: usize, parent_idx: usize, node_idx: usize, acc_builder: &mut AccessorBuilder) {
        if parent_idx != node_idx && gltf.nodes[node_idx].skin.is_some() {
            // Remove TRS
            /*gltf.nodes[node_idx].translation = None;
            gltf.nodes[node_idx].rotation = None;
            gltf.nodes[node_idx].scale = None;*/

            // Move from old parent to scene
            //let children = gltf.nodes[parent_idx].children.as_mut().unwrap();
            gltf.nodes[parent_idx].children.as_mut().unwrap().retain(|c| !c.value().eq(&node_idx));
            gltf.scenes[scene_idx].nodes.push(json::Index::new(node_idx as u32));

            if gltf.nodes[node_idx].mesh.is_some() {
                // Move node transforms
                gltf.nodes[node_idx].translation = None;
                gltf.nodes[node_idx].rotation = None;
                gltf.nodes[node_idx].scale = None;

                // Translate mesh coords
                let pos_coords = gltf
                    .nodes[node_idx]
                    .name
                    .as_ref()
                    .and_then(|n| acc_builder.get_array_by_name_mut::<f32, 3>(&format!("{n}_pos"), BufferType::Mesh));

                if let Some(pos_coords) = pos_coords {
                    let local_matrix =  gltf
                        .nodes[node_idx]
                        .name
                        .as_ref()
                        .and_then(|n| self.get_transform(n))
                        .map(|trans| {
                            let is_root = self.is_transform_root(trans);

                            if is_root {
                                let m = trans.get_world_xfm();

                                let nam = na::Matrix4::new(
                                    // Column-major order...
                                    m.m11, m.m21, m.m31, m.m41,
                                    m.m12, m.m22, m.m32, m.m42,
                                    m.m13, m.m23, m.m33, m.m43,
                                    m.m14, m.m24, m.m34, m.m44
                                );

                                println!("{} is root!", trans.get_name()); // TODO: Remove line

                                nam //* super::MILOSPACE_TO_GLSPACE
                            } else {
                                /*let wm = {
                                    let m = trans.get_world_xfm();
                                    na::Matrix4::new(
                                        // Column-major order...
                                        m.m11, m.m21, m.m31, m.m41,
                                        m.m12, m.m22, m.m32, m.m42,
                                        m.m13, m.m23, m.m33, m.m43,
                                        m.m14, m.m24, m.m34, m.m44
                                    )
                                };*/

                                let m = trans.get_local_xfm();

                                /*wm * na::Matrix4::new(
                                    // Column-major order...
                                    m.m11, m.m21, m.m31, m.m41,
                                    m.m12, m.m22, m.m32, m.m42,
                                    m.m13, m.m23, m.m33, m.m43,
                                    m.m14, m.m24, m.m34, m.m44
                                ).try_inverse().unwrap_or_default()*/

                                na::Matrix4::new(
                                    // Column-major order...
                                    m.m11, m.m21, m.m31, m.m41,
                                    m.m12, m.m22, m.m32, m.m42,
                                    m.m13, m.m23, m.m33, m.m43,
                                    m.m14, m.m24, m.m34, m.m44
                                )
                            }
                        })
                        .unwrap_or_default();

                    for [x, y, z] in pos_coords {
                        let v = local_matrix.transform_vector(&na::Vector3::new(*x, *y, *z));

                        *x = *v.get(0).unwrap();
                        *y = *v.get(1).unwrap();
                        *z = *v.get(2).unwrap();
                    }

                    // TODO: Come up with better way to share name
                    let mesh_name = gltf
                        .nodes[node_idx]
                        .name
                        .as_ref()
                        .unwrap();

                    acc_builder.recalc_min_max_values::<f32, 3>(&format!("{mesh_name}_pos"), BufferType::Mesh);
                }
            }

            // Remove children if empty
            let children_empty = gltf.nodes[parent_idx].children.as_ref().map(|c| c.is_empty()).unwrap_or_default();
            if children_empty {
                gltf.nodes[parent_idx].children.take();
            }
        }

        // Process children
        let child_count = gltf
            .nodes
            .get(node_idx)
            .and_then(|n| n.children.as_ref().map(|c| c.len()))
            .unwrap_or_default();

        for i in (0..child_count).rev() {
            let child_node_idx = gltf
                .nodes
                .get(node_idx)
                .and_then(|n| n.children.as_ref().and_then(|c| c.get(i)))
                .map(|c| c.value())
                .unwrap();

            self.move_skinned_mesh_to_skeleton_root(gltf, scene_idx, node_idx, child_node_idx, acc_builder);
        }
    }

    /*fn update_meshes_with_skins(&self, gltf: &mut json::Root, node: usize, skin: usize, meshes: &) {

    }*/

    fn build_binary(&self, gltf: &mut json::Root, acc_builder: AccessorBuilder) {
        // Write as external file
        let output_dir = self.settings.output_dir.as_path();
        super::create_dir_if_not_exists(output_dir).unwrap();

        let basename = self.get_basename();
        let filename = format!("{basename}.bin");
        let bin_path = output_dir.join(&filename);

        let (accessors, views, buffer, data) = acc_builder.generate(&filename);

        let mut writer = std::fs::File::create(&bin_path).unwrap();
        writer.write_all(&data).unwrap();

        println!("Wrote \"{filename}\"");

        /*buffer.uri = {
            use base64::{Engine as _, engine::{self, general_purpose}, alphabet};

            let mut str_data = String::from("data:application/octet-stream;base64,");
            general_purpose::STANDARD.encode_string(&data, &mut str_data);

            Some(str_data)
        };*/

        gltf.accessors = accessors;
        gltf.buffers = vec![buffer];
        gltf.buffer_views = views;
    }

    pub fn process(&mut self) -> Result<(), Box<dyn Error>> {
        let mut gltf = json::Root {
            asset: json::Asset {
                generator: Some(format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))),
                ..Default::default()
            },
            ..Default::default()
        };

        self.map_objects();

        let children = self.find_node_children();
        let root_nodes = self.get_root_nodes(&children);

        let image_indices = self.process_textures(&mut gltf);
        let mat_indices = self.process_materials(&mut gltf, &image_indices);

        let scene_nodes = root_nodes
            .into_iter()
            .map(|n| self.process_node(&mut gltf, n, &children, 0))
            .collect::<Vec<_>>();

        gltf.scene = Some(json::Index::new(0));
        gltf.scenes = vec![
            json::Scene {
                extensions: None,
                extras: Default::default(),
                name: None,
                nodes: scene_nodes
                    .into_iter()
                    .map(|i| json::Index::new(i as u32))
                    .collect(),
            }
        ];

        let mut acc_builder = AccessorBuilder::new();
        let joint_indices = self.find_skins(&mut gltf, &mut acc_builder);

        let mesh_indices = self.process_meshes(&mut gltf, &mut acc_builder, &mat_indices);

        self.process_animations(&mut gltf, &mut acc_builder);

        self.final_process_nodes(&mut gltf, &mesh_indices, &joint_indices, &mut acc_builder);

        // Write binary data
        self.build_binary(&mut gltf, acc_builder);
        //self.process_accessor_data(&mut gltf);

        self.gltf = gltf;

        /*self.gltf = json::Root {
            images,
            nodes: map_bones_to_nodes(dir_name, &bones),
            scene: Some(json::Index::new(0)),
            scenes: vec![
                json::Scene {
                    extensions: None,
                    extras: Default::default(),
                    name: None,
                    nodes: vec![json::Index::new(0)],
                }
            ],
            skins: vec![
                json::Skin {
                    extensions: None,
                    extras: Default::default(),
                    inverse_bind_matrices: None,
                    joints: joints,
                    name: None,
                    skeleton: Some(json::Index::new(0))
                }
            ],
            ..Default::default()
        };*/

        Ok(())
    }

    pub fn save_to_fs(&self) -> Result<(), Box<dyn Error>> {
        // Create output dir
        let output_dir = self.settings.output_dir.as_path();
        super::create_dir_if_not_exists(output_dir)?;

        // TODO: Replace
        /*let (obj_dir, sys_info) = self
            .object_dirs
            .iter()
            .map(|(o, _, info)| (o.as_ref(), info))
            .next()
            .unwrap();

        export_object_dir_to_gltf(obj_dir, output_dir, sys_info);*/

        // Write gltf json
        let basename = self.get_basename();
        let gltf_filename = format!("{basename}.gltf");
        let gltf_path = output_dir.join(&gltf_filename);
        let writer = std::fs::File::create(&gltf_path).expect("I/O error");
        json::serialize::to_writer_pretty(writer, &self.gltf).expect("Serialization error");

        println!("Wrote \"{gltf_filename}\"");

        Ok(())
    }

    fn find_node_children<'a>(&'a self) -> HashMap<&'a str, Vec<&'a str>> {
        // Use gh1-style child hierarchy first
        /*let (legacy_node_map, legacy_children) = self.transforms
            .values()
            .map(|t| &t.object as &dyn Trans)
            .chain(self.groups.values().map(|g| &g.object as &dyn Trans))
            .chain(self.meshes.values().map(|m| &m.object as &dyn Trans))
            .filter(|t| !t.get_trans_objects().is_empty())
            .fold((HashMap::new(), HashSet::new()), |(mut map, mut ch_set), t| {
                let parent = t.get_name().as_str();
                let children = t.get_trans_objects()
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>();

                for child in children.iter() {
                    ch_set.insert(*child);
                }

                if t.get_name() != t.get_parent() {
                    //println!("WARN: Object \"{}\", doesn't match object \"{}\"", t.get_name(), t.get_parent());
                    println!("{} : {}", t.get_name(), t.get_parent());
                }

                map.insert(parent, children);
                (map, ch_set)
            });*/

        let mut node_map = self.transforms
            .values()
            .map(|t| &t.object as &dyn Trans)
            .chain(self.groups.values().map(|g| &g.object as &dyn Trans))
            .chain(self.meshes.values().map(|m| &m.object as &dyn Trans))
            .fold(HashMap::new(), |mut acc, b| {
                if b.get_parent().eq(b.get_name()) || b.get_parent().is_empty() {
                    // If bone references self, ignore
                    return acc;
                }

                let parent = b.get_parent().as_str();
                let name = b.get_name().as_str();

                acc
                    .entry(parent)
                    .and_modify(|e: &mut Vec<&'a str>| e.push(name))
                    .or_insert_with(|| vec![name]);

                acc
            });

        // Sort children
        node_map.values_mut().for_each(|ch| ch.sort());
        node_map
    }

    fn get_root_nodes<'a>(&'a self, node_map: &HashMap<&'a str, Vec<&'a str>>) -> Vec<&'a str> {
        let children = node_map
            .values()
            .flatten()
            .map(|s| *s)
            .collect::<HashSet<_>>();

        // Anything not in child map is considered root
        self.dirs_rc
            .iter()
            .map(|d| match &d.as_ref().dir {
                ObjectDir::ObjectDir(dir) => dir.name.as_str()
            })
            .chain(self.transforms.values().map(|t| t.object.get_name().as_str()))
            .chain(self.groups.values().map(|g| g.object.get_name().as_str()))
            .chain(self.meshes.values().map(|m| m.object.get_name().as_str()))
            .filter(|s| !s.is_empty() && !children.contains(s))
            .sorted()
            .collect()
    }

    fn is_transform_root(&self, trans: &dyn Trans) -> bool {
        if trans.get_parent().is_empty() || trans.get_name().eq(trans.get_parent()) {
            return true;
        }

        self.dirs_rc
            .iter()
            .any(|d| match &d.as_ref().dir {
                ObjectDir::ObjectDir(dir) => dir.name.as_str().eq(trans.get_parent())
            })
    }

    fn process_animations(&self, gltf: &mut json::Root, acc_builder: &mut AccessorBuilder) {
        let mut animations = Vec::new();
    
        // Map indices of all named nodes
        let node_map = gltf
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| n.name.as_ref().and_then(|s| Some((s.to_owned(), i))))
            .collect::<HashMap<_, _>>();

        // Get anims in groups
        let mut groups = self
            .groups
            .values()
            .map(|g| (
                &g.object,
                g.object
                    .get_objects()
                    .iter()
                    .filter_map(|o| self.trans_anims.get(o).map(|t| &t.object))
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>();

        // Sort groups by name
        groups.sort_by(|(a, _), (b, _)| a.get_name().cmp(b.get_name()));

        for (group, mut anims) in groups {
            // Sort anims by name
            anims.sort_by(|a, b| a.get_name().cmp(b.get_name()));

            // Group anims by target
            let grouped_anims = anims
                .iter()
                .filter(|a| !a.trans_object.is_empty()
                    && (!a.rot_keys.is_empty() || !a.trans_keys.is_empty() || !a.scale_keys.is_empty())
                    && node_map.contains_key(&a.trans_object) // Remove anims with unavailable nodes
                )
                .map(|a| (&a.trans_object, a))
                .fold(HashMap::new(), |mut acc, (target, anim)| {
                    acc
                        .entry(target)
                        .and_modify(|e : &mut Vec<_>| e.push(*anim))
                        .or_insert(vec![*anim]);
                    
                    acc
                });

            let mut channels = Vec::new();
            let mut samplers = Vec::new();

            for target_name in grouped_anims.keys().sorted() {
                // Missing nodes already filtered out above
                let node_idx = node_map.get(*target_name).map(|i| *i).unwrap();

                for anim in grouped_anims.get(target_name).unwrap().iter() {
                    // Add translations
                    if !anim.trans_keys.is_empty() {
                        let input_idx = acc_builder.add_scalar(
                            format!("{}_translation_input", anim.get_name()),
                            anim.trans_keys.iter().map(|k| k.pos),
                            BufferType::Animation
                        ).unwrap();

                        let output_idx = acc_builder.add_array(
                            format!("{}_translation_output", anim.get_name()),
                            anim.trans_keys.iter().map(|k| [k.value.x, k.value.y, k.value.z]),
                            BufferType::Animation
                        ).unwrap();

                        channels.push(json::animation::Channel {
                            sampler: json::Index::new(samplers.len() as u32),
                            target: json::animation::Target {
                                node: json::Index::new(node_idx as u32),
                                path: json::validation::Checked::Valid(json::animation::Property::Translation),
                                extensions: None,
                                extras: Default::default()
                            },
                            extensions: None,
                                extras: Default::default()
                        });

                        samplers.push(json::animation::Sampler {
                            input: json::Index::new(input_idx as u32),
                            output: json::Index::new(output_idx as u32),
                            interpolation: json::validation::Checked::Valid(json::animation::Interpolation::Linear),
                            extensions: None,
                            extras: Default::default()
                        });
                    }

                    // Add rotations
                    if !anim.rot_keys.is_empty() {
                        let input_idx = acc_builder.add_scalar(
                            format!("{}_rotation_input", anim.get_name()),
                            anim.rot_keys.iter().map(|k| k.pos),
                            BufferType::Animation
                        ).unwrap();

                        let output_idx = acc_builder.add_array(
                            format!("{}_rotation_output", anim.get_name()),
                            anim.rot_keys.iter().map(|k| [k.value.x, k.value.y, k.value.z, k.value.w]),
                            BufferType::Animation
                        ).unwrap();

                        channels.push(json::animation::Channel {
                            sampler: json::Index::new(samplers.len() as u32),
                            target: json::animation::Target {
                                node: json::Index::new(node_idx as u32),
                                path: json::validation::Checked::Valid(json::animation::Property::Rotation),
                                extensions: None,
                                extras: Default::default()
                            },
                            extensions: None,
                                extras: Default::default()
                        });

                        samplers.push(json::animation::Sampler {
                            input: json::Index::new(input_idx as u32),
                            output: json::Index::new(output_idx as u32),
                            interpolation: json::validation::Checked::Valid(json::animation::Interpolation::Linear),
                            extensions: None,
                            extras: Default::default()
                        });
                    }

                    // Add scales
                    if !anim.scale_keys.is_empty() {
                        let input_idx = acc_builder.add_scalar(
                            format!("{}_scale_input", anim.get_name()),
                            anim.scale_keys.iter().map(|k| k.pos),
                            BufferType::Animation
                        ).unwrap();

                        let output_idx = acc_builder.add_array(
                            format!("{}_scale_output", anim.get_name()),
                            anim.scale_keys.iter().map(|k| [k.value.x, k.value.y, k.value.z]),
                            BufferType::Animation
                        ).unwrap();

                        channels.push(json::animation::Channel {
                            sampler: json::Index::new(samplers.len() as u32),
                            target: json::animation::Target {
                                node: json::Index::new(node_idx as u32),
                                path: json::validation::Checked::Valid(json::animation::Property::Scale),
                                extensions: None,
                                extras: Default::default()
                            },
                            extensions: None,
                                extras: Default::default()
                        });

                        samplers.push(json::animation::Sampler {
                            input: json::Index::new(input_idx as u32),
                            output: json::Index::new(output_idx as u32),
                            interpolation: json::validation::Checked::Valid(json::animation::Interpolation::Linear),
                            extensions: None,
                            extras: Default::default()
                        });
                    }
                }
            }

            if samplers.is_empty() || channels.is_empty() {
                // Don't add if no anims found
                continue;
            }

            animations.push(json::Animation {
                name: Some(group.get_name().to_owned()),
                channels,
                samplers,
                extensions: None,
                extras: Default::default()
            });
        }

        // Get char clip anims
        let mut char_clips = self
            .char_clip_samples
            .values()
            .map(|c| (&c.object, &c.parent.info))
            .collect::<Vec<_>>();

        // Sort clips by name
        char_clips.sort_by(|(a, _), (b, _)| a.get_name().cmp(b.get_name()));

        /*let char_clip_samples = char_clips
            .into_iter()
            .flat_map(|(ccs, info)| [&ccs.full, &ccs.one]
                .into_iter()
                .flat_map(|cbs| cbs
                    .decode_samples(info)
                    .into_inter()
                )
            )
            .collect::<Vec<_>>();*/

        let default_frames = vec![0.0];

        for (char_clip, info) in char_clips {
            let clip_name = char_clip.get_name();

            let mut channels = Vec::new();
            let mut samplers = Vec::new();

            // TODO: Delete after testing
            let _filtered_bone_names = HashSet::from([
                //"bone_neck.mesh"
                "bone_spine1.mesh",
                "bone_spine2.mesh",
                "bone_spine3.mesh",
            ]);

            // TODO: Decode at earlier step...
            let bone_samples = [&char_clip.one, &char_clip.full]
                .iter()
                .flat_map(|cbs| cbs.decode_samples(info)
                    .into_iter()
                    .map(|s| (s, if !cbs.frames.is_empty() { &cbs.frames } else { &default_frames })))
                //.filter(|(b, _)| filtered_bone_names.contains(b.symbol.as_str()))
                .collect::<Vec<_>>();

            for (mut bone, _frames) in bone_samples {
                let bone_name = bone.symbol.as_str();
                let Some(node_idx) = node_map.get(bone_name).map(|i| *i) else {
                    continue;
                };

                // Get existing matrix for node
                let node = &gltf.nodes[node_idx];

                let node_trans = node.translation
                    .map(na::Vector3::from)
                    .unwrap_or_else(na::Vector3::zeros);

                let node_rotate = node.rotation
                    .map(|json::scene::UnitQuaternion([x, y, z, w])|
                        na::UnitQuaternion::from_quaternion(
                            na::Quaternion::new(w, x, y, z)
                        )
                    )
                    .unwrap_or_else(|| na::UnitQuaternion::identity());

                // TODO: Add scaling transform samples...
                let _node_scale = node.scale
                    .map(na::Vector3::from)
                    .unwrap_or_else(|| na::Vector3::from_element(1.0));

                // Compute samples as matrices
                //let translate_samples = bone.pos.take().map(|(pw, p)| p.into_iter().map(|v| na::Matrix4::new_translation(&na::Vector3::new(v.x, v.y, v.z))));
                //let translate_samples = bone.quat.take().map(|(qw, q)| q.into_iter().map(|v| na::Quaternion::new(v.x, v.y, v.z, v.w)));
                //let mat = na::Matrix4::new_translation(&na::Vector3::new(1.0, 1.0, 1.0));

                /*let mut samples = Vec::new();

                // Process translations (.pos)
                if let Some((w, positions)) = bone.pos.take() {
                    for (i, v) in positions.into_iter().enumerate() {
                        let mat = match samples.get_mut(i) {
                            Some(m) => m,
                            _ => {
                                samples.push(node_matrix);
                                samples.last_mut().unwrap()
                            }
                        };

                        mat.append_translation_mut(&na::Vector3::new(v.x * w, v.y * w, v.z * w));
                    }
                }

                // Process rotations (.quat)

                // Process rotations (.rotz)

                // Add matrix samples
                let input_idx = acc_builder.add_scalar(
                    format!("{}_{}_matrix_input", clip_name, bone_name),
                    //frames.iter().map(|f| *f)
                    samples.iter().enumerate().map(|(i, _)| i as f32)
                ).unwrap();

                let output_idx = acc_builder.add_array(
                    format!("{}_{}_matrix_output", clip_name, bone_name),
                    samples.into_iter().map(|m| [
                        m[0],
                        m[1],
                        m[2],
                        m[3],
                        m[4],
                        m[5],
                        m[6],
                        m[7],
                        m[8],
                        m[9],
                        m[10],
                        m[11],
                        m[12],
                        m[13],
                        m[14],
                        m[15],
                    ])
                ).unwrap();

                channels.push(json::animation::Channel {
                    sampler: json::Index::new(samplers.len() as u32),
                    target: json::animation::Target {
                        node: json::Index::new(node_idx as u32),
                        path: json::validation::Checked::Valid(json::animation::Property::Translation),
                        extensions: None,
                        extras: Default::default()
                    },
                    extensions: None,
                        extras: Default::default()
                });

                samplers.push(json::animation::Sampler {
                    input: json::Index::new(input_idx as u32),
                    output: json::Index::new(output_idx as u32),
                    interpolation: json::validation::Checked::Valid(json::animation::Interpolation::Linear),
                    extensions: None,
                    extras: Default::default()
                });*/

                const FPS: f32 = 1. / 30.;

                // Add translations (.pos)
                if let Some((w, samples)) = bone.pos.take() {
                    let input_idx = acc_builder.add_scalar(
                        format!("{}_{}_translation_input", clip_name, bone_name),
                        //frames.iter().map(|f| *f)
                        samples.iter().enumerate().map(|(i, _)| (i as f32) * FPS),
                        BufferType::Animation
                    ).unwrap();

                    let output_idx = acc_builder.add_array(
                        format!("{}_{}_translation_output", clip_name, bone_name),
                        samples.into_iter().map(|s| {
                            let v = na::Vector3::new(s.x * w, s.y * w, s.z * w);

                            [v.x, v.y, v.z]
                        }),
                        BufferType::Animation
                    ).unwrap();

                    channels.push(json::animation::Channel {
                        sampler: json::Index::new(samplers.len() as u32),
                        target: json::animation::Target {
                            node: json::Index::new(node_idx as u32),
                            path: json::validation::Checked::Valid(json::animation::Property::Translation),
                            extensions: None,
                            extras: Default::default()
                        },
                        extensions: None,
                            extras: Default::default()
                    });
    
                    samplers.push(json::animation::Sampler {
                        input: json::Index::new(input_idx as u32),
                        output: json::Index::new(output_idx as u32),
                        interpolation: json::validation::Checked::Valid(json::animation::Interpolation::Linear),
                        extensions: None,
                        extras: Default::default()
                    });
                }
                // Don't add empty data if sample not found...
                // Targets can be shared between one/full anims and split into pos/rot/scale elements
                // TODO: Double check this...
                else {
                    // Add empty pos sample
                    let input_idx = acc_builder.add_scalar(
                        format!("{}_{}_translation_input", clip_name, bone_name),
                        //frames.iter().map(|f| *f)
                        vec![0.0],
                        BufferType::Animation
                    ).unwrap();

                    let output_idx = acc_builder.add_array(
                        format!("{}_{}_translation_output", clip_name, bone_name),
                        {
                            let v = node_trans;
                            vec![[v.x, v.y, v.z]]
                        },
                        BufferType::Animation
                    ).unwrap();

                    channels.push(json::animation::Channel {
                        sampler: json::Index::new(samplers.len() as u32),
                        target: json::animation::Target {
                            node: json::Index::new(node_idx as u32),
                            path: json::validation::Checked::Valid(json::animation::Property::Translation),
                            extensions: None,
                            extras: Default::default()
                        },
                        extensions: None,
                            extras: Default::default()
                    });
    
                    samplers.push(json::animation::Sampler {
                        input: json::Index::new(input_idx as u32),
                        output: json::Index::new(output_idx as u32),
                        interpolation: json::validation::Checked::Valid(json::animation::Interpolation::Linear),
                        extensions: None,
                        extras: Default::default()
                    });
                }

                /*let mut rotation_samples = [
                    bone.quat
                        .take()
                        .map(|(w, samples)| samples.into_iter().map(|s| {
                            na::UnitQuaternion::from_quaternion(
                                na::Quaternion::new(
                                    s.x * w,
                                    s.y * w,
                                    s.z * w,
                                    s.w * w,
                            ))
                        })
                        .by_ref()
                        .collect::<Vec<_>>()
                    ).unwrap_or_default(),
                    bone.rotz
                        .take()
                        .map(|(w, samples)| samples.into_iter().map(|s| {
                            na::UnitQuaternion::from_axis_angle(
                                &na::Vector3::z_axis(),
                                std::f32::consts::PI * (s * w)
                            )
                        })
                        .collect::<Vec<_>>()
                    )
                    .unwrap_or_default()
                ];*/

                /*let rotation_sample_count = match (bone.quat.as_ref().map(|(_, s)| s.len()), bone.rotz.as_ref().map(|(_, s)| s.len())) {
                    (Some(a), Some(b)) => a.max(b),
                    (Some(a), _) => a,
                    (_, Some(b)) => b,
                    _ => bone.pos.as_ref().map(|(_, s)| s.len()).unwrap_or_default()
                };*/

                let rotation_sample_count = [
                    bone.quat.as_ref().map(|(_, s)| s.len()),
                    bone.rotz.as_ref().map(|(_, s)| s.len()),
                    bone.pos.as_ref().map(|(_, s)| s.len())
                ]
                .into_iter()
                .filter_map(|f| f)
                .max()
                .unwrap_or_default();

                // Combined rotations
                let mut rotation_samples = (0..rotation_sample_count)
                    .map(|_| node_rotate)
                    /*.map(|_| {
                        let q = rotation.as_vector();
                        na::Quaternion::new(q[3], q[0], q[1], q[2])
                    })*/
                    .collect::<Vec<_>>();

                // Add rotations (.quat)
                if let Some((w, samples)) = bone.quat.take() {
                    for (i, s) in samples.into_iter().enumerate() {
                        let rot =  &mut rotation_samples[i];

                        let q = na::Quaternion::new(
                            s.w * w,
                            s.x * w,
                            s.y * w,
                            s.z * w,
                        );

                        //let q = na::UnitQuaternion::from

                        //*rot = *rot * q;

                        //*rot = rot.rotation_to(&na::UnitQuaternion::from_quaternion(q));
                        *rot = na::UnitQuaternion::from_quaternion(q);
                        //*rot = na::UnitQuaternion::from_quaternion(rot.normalize());
                    }
                }

                // Add rotations (.rotz)
                if let Some((w, samples)) = bone.rotz.take() {
                    for (i, z) in samples.into_iter().enumerate() {
                        let rot =  &mut rotation_samples[i];

                        let q = na::UnitQuaternion::from_axis_angle(
                            &na::Vector3::z_axis(),
                            std::f32::consts::PI * (z * w)
                        );

                        *rot *= q;
                    }
                }

                // Add all rotations
                // TODO: Add empty rotation sample?
                if rotation_samples.len() > 0 {
                    let input_idx = acc_builder.add_scalar(
                        format!("{}_{}_rotation_input", clip_name, bone_name),
                        //frames.iter().map(|f| *f)
                        rotation_samples.iter().enumerate().map(|(i, _)| (i as f32) * FPS),
                        BufferType::Animation
                    ).unwrap();

                    let output_idx = acc_builder.add_array(
                        format!("{}_{}_rotation_output", clip_name, bone_name),
                        rotation_samples.into_iter().map(|s| [s[0], s[1], s[2], s[3]]),
                        BufferType::Animation
                    ).unwrap();

                    channels.push(json::animation::Channel {
                        sampler: json::Index::new(samplers.len() as u32),
                        target: json::animation::Target {
                            node: json::Index::new(node_idx as u32),
                            path: json::validation::Checked::Valid(json::animation::Property::Rotation),
                            extensions: None,
                            extras: Default::default()
                        },
                        extensions: None,
                            extras: Default::default()
                    });

                    samplers.push(json::animation::Sampler {
                        input: json::Index::new(input_idx as u32),
                        output: json::Index::new(output_idx as u32),
                        interpolation: json::validation::Checked::Valid(json::animation::Interpolation::Linear),
                        extensions: None,
                        extras: Default::default()
                    });
                }

                // Add scales (.scale)
            }

            if samplers.is_empty() || channels.is_empty() {
                // Don't add if no anims found
                continue;
            }

            animations.push(json::Animation {
                name: Some(clip_name.to_owned()),
                channels,
                samplers,
                extensions: None,
                extras: Default::default()
            });
        }

        gltf.animations = animations;
    }
}

fn decompose_trs(mat: na::Matrix4<f32>) -> (na::Vector3<f32>, na::UnitQuaternion<f32>, na::Vector3<f32>) {
    // Decompose matrix to T*R*S
    let translate = mat.column(3).xyz();
    let rotation = na::UnitQuaternion::from_matrix(&mat.fixed_view::<3, 3>(0, 0).into());

    let scale = na::Vector3::new(
        mat.column(0).magnitude(),
        mat.column(1).magnitude(),
        mat.column(2).magnitude(),
    );

    (translate, rotation, scale)
}

fn decompose_trs_with_milo_coords(mut mat: na::Matrix4<f32>) -> (na::Vector3<f32>, na::UnitQuaternion<f32>, na::Vector3<f32>) {
    mat = mat * super::MILOSPACE_TO_GLSPACE;

    // Decompose matrix to T*R*S
    let translate = mat.column(3).xyz();
    //let cc = node_matrix.fixed_view::<3, 3>(0, 0);
    //let rot = na::UnitQuaternion::from_matrix(&cc.into());
    let rotation = na::UnitQuaternion::from_matrix(&mat.fixed_view::<3, 3>(0, 0).into());
    //let scale = node_matrix.column(0).xyz().component_mul(&node_matrix.column(1).xyz()).component_mul(&node_matrix.column(2).xyz());
    let scale = na::Vector3::new(
        mat.column(0).magnitude(),
        mat.column(1).magnitude(),
        mat.column(2).magnitude(),
    );

    let q = {
        let q = rotation.to_rotation_matrix();
        let q4 = na::Matrix4x3::identity() * (q * na::Matrix3x4::identity());

        let m = q4 * super::MILOSPACE_TO_GLSPACE;
        na::UnitQuaternion::from_matrix(&m.fixed_view::<3, 3>(0, 0).into())
    };

    (
        super::MILOSPACE_TO_GLSPACE.transform_vector(&translate),
        q,
        super::MILOSPACE_TO_GLSPACE.transform_vector(&scale)
    )
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use super::*;

    #[rstest]
    fn accessor_builder_test() {
        let mut acc_builder = AccessorBuilder::new();

        //acc_builder.add_array_f32([[0.0f32, 0.1f32, 0.2f32]]);

        acc_builder.add_array("", [[0.0f32, 0.1f32, 0.2f32]], BufferType::Animation);
        //acc_builder.add("", [0.0f32, 0.1f32, 0.2f32]);

        //assert!(false);
    }

    #[rstest]
    fn decompose_trs_identity_test() {
        let mat = na::Matrix4::identity();

        let (trans, rotate, scale) = decompose_trs(mat);

        assert_eq!(na::Vector3::zeros(), trans);
        assert_eq!(na::UnitQuaternion::identity(), rotate);
        assert_eq!(na::Vector3::from_element(1.0), scale);
    }

    #[rstest]
    #[case([0.0, 0.0, 0.0])]
    #[case([1.0, 2.0, 3.0])]
    #[case([-1.0, 2.0, -10.0])]
    fn decompose_trs_with_translation_test(#[case] input_trans: [f32; 3]) {
        let [tx, ty, tz] = input_trans;

        let mat = na::Matrix4::new(
            1.0, 0.0, 0.0,  tx,
            0.0, 1.0, 0.0,  ty,
            0.0, 0.0, 1.0,  tz,
            0.0, 0.0, 0.0, 1.0,
        );

        let (trans, rotate, scale) = decompose_trs(mat);

        assert_eq!(na::Vector3::from(input_trans), trans);
        assert_eq!(na::UnitQuaternion::identity(), rotate);
        assert_eq!(na::Vector3::from_element(1.0), scale);
    }

    #[rstest]
    #[case([1.0 ,  1.0,  1.0])]
    #[case([20.0, 20.0, 20.0])]
    #[case([50.0, 20.0, 50.0])]
    #[case([10.0, 20.0, 30.0])]
    fn decompose_trs_with_scale_test(#[case] input_scale: [f32; 3]) {
        let [sx, sy, sz] = input_scale;

        let mat = na::Matrix4::new(
             sx, 0.0, 0.0, 0.0,
            0.0,  sy, 0.0, 0.0,
            0.0, 0.0,  sz, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        let (trans, rotate, scale) = decompose_trs(mat);

        assert_eq!(na::Vector3::zeros(), trans);
        assert_eq!(na::UnitQuaternion::identity(), rotate);
        assert_eq!(na::Vector3::new(sx, sy, sz), scale);
    }

    #[rstest]
    #[case(90.0, [0.0, 0.0, 0.7071068, 0.7071067])] // Both should be 0.7071068. Precision issue?
    fn decompose_trs_rotz_test(#[case] input_deg: f32, #[case] expected_result: [f32; 4]) {
        let rad = (input_deg * std::f32::consts::PI) / 180.0;
        let [i, j, k, w] = expected_result;

        // Rotate on z-axis
        let mat = na::Matrix4::from_axis_angle(&na::Vector3::z_axis(), rad);

        let (trans, rotate, scale) = decompose_trs(mat);

        assert_eq!(na::Vector3::zeros(), trans);
        assert_eq!(na::UnitQuaternion::from_quaternion(na::Quaternion::new(w, i, j, k)), rotate);
        assert_eq!(na::Vector3::from_element(1.0), scale);
    }

    #[rstest]
    #[case(90.0, [1.0 ,  1.0,  1.0], [0.0, 0.0, 0.7071068, 0.7071067])]
    #[case(90.0, [20.0, 20.0, 20.0], [0.0, 0.0, 0.7071068, 0.7071067])]
    #[case(90.0, [50.0, 20.0, 50.0], [0.0, 0.0, 0.7071067, 0.7071068])]
    #[case(90.0, [10.0, 20.0, 30.0], [0.0, 0.0, 0.7071067, 0.7071067])]
    fn decompose_trs_rotz_with_scale_test(#[case] input_deg: f32, #[case] input_scale: [f32; 3], #[case] expected_result: [f32; 4]) {
        let rad = (input_deg * std::f32::consts::PI) / 180.0;
        let [sx, sy, sz] = input_scale;
        let [i, j, k, w] = expected_result;

        let expected_scale = na::Vector3::new(sx, sy, sz);

        // Rotate on z-axis + scale
        let mut mat = na::Matrix4::from_axis_angle(&na::Vector3::z_axis(), rad);
        mat *= na::Matrix4::new_nonuniform_scaling(&expected_scale);

        let (trans, rotate, scale) = decompose_trs(mat);

        assert_eq!(na::Vector3::zeros(), trans);
        assert_eq!(na::UnitQuaternion::from_quaternion(na::Quaternion::new(w, i, j, k)), rotate);
        assert_eq!(expected_scale, scale);
    }
}