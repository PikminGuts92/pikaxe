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
use crate::SystemInfo;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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
    char_clip_samples: Vec<CharClipSamples>,
}

impl MiloAssets {
    pub fn dump_to_directory<T>(&self, out_dir: T, info: &SystemInfo) -> Result<(), Box<dyn std::error::Error>> where T: AsRef<Path> {
        // Create output dir
        super::create_dir_if_not_exists(&out_dir)?;

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

        // TODO: Compute global matrix of each bone node for use in converting local gltf space to milo space
        // TODO: Allow char clips for non-bones (i.e. bone_door in gh2)
        // Just get bones for first skin
        let bone_ids = self
            .document
            .skins()
            .map(|s| s
                .joints()
                .map(|j| j.index())
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

                // Parse translation animations
                if let Some(channel) = pos {
                    let reader = channel.reader(|buffer| Some(&self.buffers[buffer.index()]));
                    //let inputs = reader.read_inputs().unwrap().collect::<Vec<_>>(); // Time input

                    let outputs = match reader.read_outputs() {
                        Some(ReadOutputs::Translations(trans)) => trans.map(|[x, y, z]| Vector3 { x, y, z }).collect(),
                        _ => panic!("Unable to read translation animations for bone {target_name}"),
                    };

                    sample.pos = Some((1.0, outputs))
                }

                // Parse rotation animations
                // TODO: How to handle rotz?
                if let Some(channel) = rot {
                    let reader = channel.reader(|buffer| Some(&self.buffers[buffer.index()]));
                    //let inputs = reader.read_inputs().unwrap().collect::<Vec<_>>(); // Time input

                    let outputs = match reader.read_outputs() {
                        Some(ReadOutputs::Rotations(rots)) => rots.into_f32().map(|[x, y, z, w]| Quat { x, y, z, w }).collect(),
                        _ => panic!("Unable to read rotation animations for bone {target_name}"),
                    };

                    sample.quat = Some((1.0, outputs))
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

                full_samples.insert(node_idx, sample);
            }

            // Compute one samples
            // Find any bone transformation w/o animation and default to rest position
            let one_samples = bone_ids
                .iter()
                .filter_map(|b| match full_samples.get(b) {
                    Some(s) if s.pos.is_some() && s.quat.is_some() => None,
                    s @ _ => { // Has 0 or some transformations
                        let (name, node) = node_map.get(b).expect("Get bone node for one anim");
                        let ([tx, ty, tz], [rx, ry, rz, rw], [_sx, _sy, _sz]) =  node.transform().decomposed();

                        Some(CharBoneSample {
                            symbol: name.to_string(),
                            pos: if s.is_none_or(|s| s.pos.is_none()) {
                                Some((1.0, vec![Vector3 { x: tx, y: ty, z: tz }]))
                            } else {
                                None
                            },
                            quat: if s.is_none_or(|s| s.quat.is_none()) {
                                Some((1.0, vec![Quat { x: rx, y: ry, z: rz, w: rw }]))
                            } else {
                                None
                            },
                            ..Default::default()
                        })
                    }
                })
                .collect();

            let mut clip = CharClipSamples {
                name: anim_name,
                one: CharBonesSamples {
                    compression: 1,
                    samples: EncodedSamples::Uncompressed(one_samples),
                    ..Default::default()
                },
                full: CharBonesSamples {
                    compression: 1,
                    samples: EncodedSamples::Uncompressed(full_samples
                        .into_values()
                        .collect()),
                    ..Default::default()
                },
                ..Default::default()
            };

            // Re-compute char bones from sample names
            for sam in [&mut clip.one, &mut clip.full] {
                sam.generate_bones_from_samples();
                sam.recompute_sizes();
            }

            assets.char_clip_samples.push(clip);
        }

        assets
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