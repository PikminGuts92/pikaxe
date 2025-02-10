use crate::SystemInfo;
use crate::model::MILOSPACE_TO_GLSPACE;
use crate::scene::*;
use gltf::animation::util::ReadOutputs;
use gltf::animation::Property;
use gltf::buffer::Data as BufferData;
use gltf::{Document, Error as GltfError, Gltf, Mesh, Primitive, Scene};
use gltf::image::{Data as ImageData, Source};
use gltf::mesh::*;
use gltf::mesh::util::*;
use gltf::json::extensions::scene::*;
use gltf::json::extensions::mesh::*;
use gltf::scene::Node;
use nalgebra as na;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const IGNORED_BONES: [&str; 2] = [
    "bone_pos_guitar.mesh",
    "spot_ui.mesh",
];

pub struct GltfImporter2 {
    source_path: PathBuf,
    document: Document,
    buffers: Vec<BufferData>,
    images: Vec<ImageData>,
    //node_names: HashMap<usize, String>,
}

#[derive(Default)]
pub struct SceneHelper {
    nodes: HashMap<usize, String>
}

#[derive(Default)]
pub struct MiloAssets {
    char_bones: Vec<CharBone>,
    char_clip_samples: Vec<CharClipSamples>,
}

fn find_chidren_of_ignored_bones(node: &Node<'_>, ignored: &[&str], is_ignored: bool) -> Vec<usize> {
    if is_ignored {
        let mut descendents = node
            .children()
            .flat_map(|c| {
                let mut descendents = find_chidren_of_ignored_bones(&c, ignored, true);

                descendents.insert(0, c.index());
                descendents
            })
            .collect::<Vec<_>>();

        descendents.insert(0, node.index());
        return descendents
    }

    let should_ignore = IGNORED_BONES
        .iter()
        .any(|b| node.name().is_some_and(|n| n.eq(*b)));

    let mut descendents = node
        .children()
        .flat_map(|c| {
            let descendents = find_chidren_of_ignored_bones(&c, ignored, should_ignore);

            //descendents.insert(0, c.index());
            descendents
        })
        .collect::<Vec<_>>();

    if should_ignore {
        descendents.insert(0, node.index());
    }

    descendents
}

impl MiloAssets {
    pub fn dump_to_directory<T>(&self, out_dir: T, info: &SystemInfo) -> Result<(), Box<dyn std::error::Error>> where T: AsRef<Path> {
        // Create output dir
        super::create_dir_if_not_exists(&out_dir)?;

        // Write char bones
        for char_bone in self.char_bones.iter() {
            let char_bone_dir = out_dir.as_ref().join("CharBone");
            super::create_dir_if_not_exists(&char_bone_dir)?;

            let char_bone_path = char_bone_dir.join(&char_bone.name);
            save_to_file(char_bone, &char_bone_path, info)?;
        }

        // Write char clip samples
        for char_clip in self.char_clip_samples.iter() {
            let char_clip_dir = out_dir.as_ref().join("CharClipSamples");
            super::create_dir_if_not_exists(&char_clip_dir)?;

            let char_clip_path = char_clip_dir.join(&char_clip.name);
            save_to_file(char_clip, &char_clip_path, info)?;
        }

        Ok(())
    }
}

impl GltfImporter2 {
    pub fn new<T>(source_path: T) -> Result<Self, GltfError> where T: AsRef<Path> {
        let (document, buffers, images) = gltf::import(&source_path)?;

        Ok(Self {
            source_path: source_path.as_ref().to_owned(),
            document: document,
            buffers,
            images,
        })
    }

    pub fn process(&self) -> MiloAssets {
        let mut assets = MiloAssets::default();

        // Only support first scene for now?

        /*
        for scene in self.document.scenes() {
            
        }*/

        let node_map = self
            .document
            .nodes()
            .filter_map(|n| n.name().map(|nm| (n.index(), (get_basename(nm), n))))
            .collect::<HashMap<_, _>>();

        // Need to look at skins and find bones
        // If bones have no anim events, add to "one" clips

        let ignored_nodes = self
            .document
            .default_scene().unwrap()
            .nodes()
            .filter(|n| n.name().is_some())
            .flat_map(|n| find_chidren_of_ignored_bones(
                &n,
                &IGNORED_BONES,
                IGNORED_BONES
                    .iter()
                    .any(|b| n.name().unwrap().eq(*b)))
            )
            .collect::<HashSet<_>>();

        // TODO: Compute global matrix of each bone node for use in converting local gltf space to milo space
        // TODO: Allow char clips for non-bones (i.e. bone_door in gh2)
        // Just get bones for first skin
        let bone_ids = self
            .document
            .skins()
            .map(|s| s
                .joints()
                .filter(|j| j.children().any(|_| true)) // Ignore if missing children (causes issues for finger03 bones)
                .map(|j| j.index())
                .filter(|j| !ignored_nodes.contains(j))
                .collect::<HashSet<_>>())
            .next()
            .unwrap_or_else(|| HashSet::new());

        if bone_ids.is_empty() {
            log::warn!("No skin with bones found!");
        }

        // Process character animations
        for anim in self.document.animations() {
            let anim_name = anim
                .name()
                .map(|n| n.to_owned())
                .unwrap_or_else(|| format!("anim_{}", anim.index()));

            // Group channels by target
            let channels = anim.channels().collect::<Vec<_>>();
            let group_channels = channels
                .iter()
                .fold(HashMap::new(), |mut acc, ch| {
                    let key = ch.target().node().index();

                    if !bone_ids.contains(&key) {
                        // Ignore anim if not for bone
                        return acc
                    }

                    acc
                        .entry(key)
                        .and_modify(|e: &mut Vec<_>| e.push(ch))
                        .or_insert_with(|| vec![ch]);

                    acc
                });

            // Look for translate, rotate, and scale events
            let mut full_samples = HashMap::new();
            let mut static_samples: HashMap<usize, (Option<Vector3>, Option<Quat>, Option<f32>)> = HashMap::new();

            for (node_idx, channels) in group_channels {
                // Ignore if node doesn't have associated name
                let Some((target_name, _)) = node_map.get(&node_idx) else {
                    log::info!("No associated name for node with index {node_idx}, skipping");
                    continue;
                };

                let (pos, rot, scale) = channels
                    .iter()
                    .fold((None, None, None), |(pos, rot, scale), c| match &c.target().property() {
                        Property::Translation => (Some(c), rot, scale),
                        Property::Rotation => (pos, Some(c), scale),
                        Property::Scale => (pos, rot, Some(c)),
                        _ => (pos, rot, scale)
                    });

                if pos.is_none() && rot.is_none() && scale.is_none() {
                    log::info!("Animation for bone {target_name} has no compatible transform, skipping");
                    continue;
                }

                let mut sample = CharBoneSample {
                    symbol: target_name.to_string(),
                    ..Default::default()
                };

                // TODO: Interpolate frames for non 30fps animations
                // Will need to use time inputs for that

                // Note: Only rotational transformations are support for bones except for root bone
                // TODO: TODO: Remove hard-coded root bone value
                let is_root = target_name.eq(&"bone_pelvis");

                // Parse translation animations
                if let Some(channel) = pos.and_then(|p| is_root.then(|| p)) {
                    let reader = channel.reader(|buffer| Some(&self.buffers[buffer.index()]));
                    //let inputs = reader.read_inputs().unwrap().collect::<Vec<_>>(); // Time input

                    let outputs = match reader.read_outputs() {
                        Some(ReadOutputs::Translations(trans)) => trans.map(|[x, y, z]| Vector3 { x, y, z }).collect::<Vec<_>>(),
                        _ => panic!("Unable to read translation animations for bone {target_name}"),
                    };

                    let first_pos = outputs[0].clone();
                    let has_changes = outputs.iter().skip(1).any(|o| !o.eq(&first_pos));

                    if has_changes {
                        sample.pos = Some((1.0, outputs));
                    } else {
                        static_samples
                            .entry(node_idx)
                            .and_modify(|(pos, _, _)| {
                                *pos = Some(first_pos.clone());
                            })
                            .or_insert_with(|| (Some(first_pos), None, None));
                    }
                }

                // Parse rotation animations
                // TODO: How to handle rotz?
                if let Some(channel) = rot {
                    let reader = channel.reader(|buffer| Some(&self.buffers[buffer.index()]));
                    //let inputs = reader.read_inputs().unwrap().collect::<Vec<_>>(); // Time input

                    let outputs = match reader.read_outputs() {
                        Some(ReadOutputs::Rotations(rots)) => rots.into_f32().map(|[x, y, z, w]| Quat { x, y, z, w }).collect::<Vec<_>>(),
                        _ => panic!("Unable to read rotation animations for bone {target_name}"),
                    };

                    let first_quat = outputs[0].clone();
                    let has_changes = outputs.iter().skip(1).any(|o| !o.eq(&first_quat));

                    if has_changes {
                        sample.quat = Some((1.0, outputs));
                    } else {
                        static_samples
                            .entry(node_idx)
                            .and_modify(|(_, quat, _)| {
                                *quat = Some(first_quat.clone());
                            })
                            .or_insert_with(|| (None, Some(first_quat), None));
                    }
                }

                // Parse scale animations
                // TODO: Don't skip scales
                /*if let Some(channel) = scale {
                    let reader = channel.reader(|buffer| Some(&self.buffers[buffer.index()]));
                    //let inputs = reader.read_inputs().unwrap().collect::<Vec<_>>(); // Time input

                    let outputs = match reader.read_outputs() {
                        Some(ReadOutputs::Scales(scales)) => scales.map(|[x, y, z]| Vector3 { x, y, z }).collect(),
                        _ => panic!("Unable to read scale animations for bone {target_name}"),
                    };

                    //sample.scale = Some((1.0, outputs))
                }*/

                if sample.pos.is_some() || sample.quat.is_some() || sample.rotz.is_some() {
                    full_samples.insert(node_idx, sample);
                }
            }

            // Compute one samples
            // Find any bone transformation w/o animation and default to rest position
            let mut one_samples = bone_ids
                .iter()
                .filter_map(|b| -> Option<CharBoneSample> { match full_samples.get(b) {
                    Some(s) if s.pos.is_some() && s.quat.is_some() => None,
                    s @ _ => { // Has 0 or some transformations
                        let (name, node) = node_map.get(b).expect("Get bone node for one anim");
                        let ([tx, ty, tz], [rx, ry, rz, rw], [_sx, _sy, _sz]) = node.transform().decomposed();

                        let (static_pos, static_quat, _static_rotz) = {
                            let static_sample = static_samples.remove(b);

                            // Ugh... can't chain with as_mut()
                            let static_pos = static_sample.as_ref().and_then(|(p, _, _)| p.clone());
                            let static_quat = static_sample.as_ref().and_then(|(_, q, _)| q.clone());
                            let static_rotz = static_sample.as_ref().and_then(|(_, _, r)| r.clone());

                            (static_pos, static_quat, static_rotz)
                        };

                        Some(CharBoneSample {
                            symbol: name.to_string(),
                            pos: None,
                            /*pos: static_pos.map_or_else(
                                || s.is_none_or(|s| s.pos.is_none()).then(|| (1.0, vec![Vector3 { x: tx, y: ty, z: tz }])),
                                |p| Some((1.0, vec![p]))),*/
                            /*pos: if s.is_none_or(|s| s.pos.is_none()) {
                                Some((1.0, vec![Vector3 { x: tx, y: ty, z: tz }]))
                            } else {
                                None
                            },*/
                            quat: static_quat.map_or_else(
                                || s.is_none_or(|s| s.quat.is_none()).then(|| (1.0, vec![Quat { x: rx, y: ry, z: rz, w: rw }])),
                                |q| Some((1.0, vec![q]))),
                            /*quat: if s.is_none_or(|s| s.quat.is_none()) {
                                Some((1.0, vec![Quat { x: rx, y: ry, z: rz, w: rw }]))
                            } else {
                                None
                            },*/
                            ..Default::default()
                        })
                    }
                }})
                .collect::<Vec<_>>();

            let mut full_samples = full_samples
                .into_values()
                .collect::<Vec<_>>();

            // Must be sorted!!!!
            // TODO: Move to CCS io
            full_samples.sort_by(|a, b| a.symbol.cmp(&b.symbol));
            one_samples.sort_by(|a, b| a.symbol.cmp(&b.symbol));

            let mut clip = CharClipSamples {
                name: anim_name,
                start_beat: 0.0,
                end_beat: 0.0,
                beats_per_sec: 2.5, // Match UI anims
                play_flags: 512, // kPlayRealTime
                blend_width: 1.0, // Match UI anims
                one: CharBonesSamples {
                    compression: 1,
                    samples: EncodedSamples::Uncompressed(one_samples),
                    ..Default::default()
                },
                full: CharBonesSamples {
                    compression: 1,
                    samples: EncodedSamples::Uncompressed(full_samples),
                    ..Default::default()
                },
                ..Default::default()
            };

            // Update end time
            let sample_count = clip.full.get_sample_count().checked_sub(1).unwrap_or_default().max(1); // Assume at least one sample because of one anims
            clip.end_beat = (sample_count as f32 / 30.0) * clip.beats_per_sec;

            // Re-compute char bones from sample names
            for sam in [&mut clip.one, &mut clip.full] {
                sam.generate_bones_from_samples();
                sam.recompute_sizes();
            }

            assets.char_clip_samples.push(clip);
        }

        assets.char_bones = self.generate_char_bones();

        assets
    }

    fn generate_char_bones(&self)-> Vec<CharBone> {
        let default_scene = self.document.default_scene().unwrap();

        let mut bones = Vec::new();
        for node in default_scene.nodes() {
            let mut result_bones = self.process_char_bone_node(&node, None, MILOSPACE_TO_GLSPACE);
            bones.append(&mut result_bones);
        }

        bones
    }

    fn process_char_bone_node(&self, node: &Node<'_>, parent_name: Option<&String>, parent_world_matrix: na::Matrix4<f32>) -> Vec<CharBone> {
        let local_matrix = na::Matrix4::from(node.transform().matrix());
        let world_matrix = parent_world_matrix * local_matrix;

        // Check if node has name and part of bones
        let mut bones = Vec::new();
        let node_name = node.name().map(|n| format!("{}.trans", get_basename(n)));
        let is_bone = node
            .name()
            .map(|n| n.ends_with(".mesh") // TODO: Case-insensitive compare
                && node.mesh().is_none()
            )
            .unwrap_or_default();

        if let Some(name) = node_name.as_ref().and_then(|n| is_bone.then(|| n)) {
            bones.push(CharBone {
                name: name.to_owned(),
                local_xfm: na_matrix_to_milo_matrix(local_matrix),
                world_xfm: na_matrix_to_milo_matrix(world_matrix),
                parent: parent_name
                    .map(|p| p.to_owned())
                    .unwrap_or_default(),
                ..Default::default()
            });
        }

        let node_name = node_name.and_then(|n| is_bone.then(|| n)); // Only pass node name if bone
        for child in node.children() {
            let mut result_bones = self.process_char_bone_node(&child, node_name.as_ref(), world_matrix);
            bones.append(&mut result_bones);
        }

        bones
    }
}

fn na_matrix_to_milo_matrix(mat: na::Matrix4<f32>) -> Matrix {
    let m = mat.as_slice();

    /*Matrix {
        // Row-major -> column-major
        m11: m[0], m21: m[1], m31: m[2], m41: m[3],
        m12: m[4], m22: m[5], m32: m[6], m42: m[7],

        m13: m[ 8], m23: m[ 9], m33: m[10], m43: m[11],
        m14: m[12], m24: m[13], m34: m[14], m44: m[15],
    }*/

    Matrix {
        m11: m[ 0],
        m12: m[ 1],
        m13: m[ 2],
        m14: m[ 3],
        m21: m[ 4],
        m22: m[ 5],
        m23: m[ 6],
        m24: m[ 7],
        m31: m[ 8],
        m32: m[ 9],
        m33: m[10],
        m34: m[11],
        m41: m[12],
        m42: m[13],
        m43: m[14],
        m44: m[15],
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use super::*;

    #[rstest]
    fn calc_world_space_transform() {
        use nalgebra as na;

        // Need to find local matrix that produces same computed world matrix
        // Gltf space
        let orig_anim_transform = na::Translation3::new(1.0, 2.0, 3.0).to_homogeneous()
            * na::Rotation3::from_axis_angle(&na::Vector3::y_axis(), std::f32::consts::FRAC_PI_2).to_homogeneous() // 90 degrees on y-axis
            * na::Scale3::new(5.0, 10.0, 15.0).to_homogeneous();

        let node_transforms = [
            crate::model::MILOSPACE_TO_GLSPACE,
            na::Rotation3::from_axis_angle(&na::Vector3::x_axis(), std::f32::consts::PI).to_homogeneous(), // 180 degrees on x-axis
            na::Translation3::new(30.0, 20.0, -40.0).to_homogeneous(),
        ];

        let gltf_parent_world_transform = node_transforms
            .iter()
            .fold(na::Matrix4::identity(), |acc, t| acc * t);

        let gltf_parent_world_transform_inverse = gltf_parent_world_transform.try_inverse().unwrap();

        // Milo space
        let new_anim_transform = gltf_parent_world_transform_inverse
            * crate::model::MILOSPACE_TO_GLSPACE
            * gltf_parent_world_transform
            * orig_anim_transform;

        // Both should equal each other
        let orig_milo_world_anim_transform = crate::model::MILOSPACE_TO_GLSPACE
            * gltf_parent_world_transform
            * orig_anim_transform;

        let new_milo_world_anim_transform = gltf_parent_world_transform * new_anim_transform;

        assert_eq!(orig_milo_world_anim_transform, new_milo_world_anim_transform);
    }
}

fn get_basename(name: &str) -> &str {
    // Get string value until dot
    name
        .split_terminator(".")
        .next()
        .unwrap_or(name)
}