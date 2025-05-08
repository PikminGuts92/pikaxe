use std::env;
use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use keyframe::{CanTween, keyframes, Keyframe, AnimationSequence, functions::Linear, functions::EaseInOut};

use pikaxe::{Platform, SystemInfo};
use pikaxe::io::*;
use pikaxe::scene::{Anim, CharBoneSample, Object, ObjectDir, PackedObject, MeshAnim, MiloObject, Trans, Vector3};

use nalgebra as na;

use rerun::external::glam;
use rerun::{
    coordinates::{Handedness, SignedAxis3},
    components::{LineStrip3D, Position3D, Radius, Scalar, ViewCoordinates},
    RecordingStream, RecordingStreamBuilder,
    time::Timeline,
};
use rerun::{Arrows3D, Points3D};

use shared::*;

#[derive(Clone, Default)]
struct Vec3Collection(Vec<Vector3>);

impl CanTween for Vec3Collection {
    fn ease(from: Self, to: Self, time: impl keyframe::num_traits::Float) -> Self {
        let (Self(from), Self(to)) = (from, to);

        let mut points = Vec::new();

        for (from, to) in from.into_iter().zip(to.into_iter()) {
            let Vector3 { x: x1, y: y1, z: z1 } = from;
            let Vector3 { x: x2, y: y2, z: z2 } = to;

            points.push(Vector3 {
                x: f32::ease(x1, x2, time),
                y: f32::ease(y1, y2, time),
                z: f32::ease(z1, z2, time)
            });
        }

        Self(points)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().skip(1).collect();

    if args.len() < 1 {
        println!("anim_preview.exe [input_milo_path]");
        return Ok(());
    }

    let milo_path = PathBuf::from(&args[0]);

    if let Some(file_name) = milo_path.file_name() {
        let file_name = file_name.to_str().unwrap_or("file");

        println!("Opening {}", file_name);
    }

    // Open milo
    let base_milo_loader = MiloLoader::from_path(milo_path)?;

    // Get mesh anims
    let mesh_anims = base_milo_loader
        .obj_dir
        .get_entries()
        .iter()
        .filter_map(|e| match e {
            Object::MeshAnim(ma) => Some(ma),
            _ => None
        })
        .collect::<Vec<_>>();

    let (rec_stream, storage) = RecordingStreamBuilder::new("anim_preview").memory()?;

    rec_stream.log_timeless(
        "world",
        &rerun::ViewCoordinates::new(
            ViewCoordinates::from_up_and_handedness(
                SignedAxis3::POSITIVE_Z,
                Handedness::Right))
    )?;

    for mesh_anim in mesh_anims {
        println!("{}", mesh_anim.get_name());
        println!("{} point keys", mesh_anim.vert_point_keys.len());

        // Collect mesh anim frames
        let frames = mesh_anim
            .vert_point_keys
            .iter()
            .map(|v| Keyframe::new(Vec3Collection(v.value.clone()), v.pos, Linear))
            .collect::<Vec<_>>();

        // Generate missing frames with linear interpolation
        let mut sequence = AnimationSequence::from(frames);
        sequence.advance_to(0.0);

        println!("Sequence length is {}", sequence.duration());

        let mut interp_frames = Vec::new();

        while !sequence.finished() {
            let new_frame = sequence.now();
            interp_frames.push(new_frame);

            sequence.advance_by(1.0);
        }

        println!("{} interpolated point keys", interp_frames.len());

        for (i, frame) in interp_frames.into_iter().enumerate() {
            let Vec3Collection(points) = frame;

            /*let strip: LineStrip3D = points
                .iter()
                .map(|Vector3 { x, y, z }| [ *x, *y, *z ])
                .collect::<Vec<_>>()
                .into();*/

            /*let mesh = RawMesh3D {
                mesh_id: MeshId::random(),
                positions: points
                    .iter()
                    .map(|Vector3 { x, y, z }| [ *x, *y, *z ])
                    .collect::<Vec<_>>()
            };*/

            let glam_points = points
                .into_iter()
                .map(|Vector3 { x, y, z }| Position3D::new(x, y, z))
                .collect::<Vec<_>>();

            // Send points to rerun
            rec_stream.set_time_sequence("frame", i as i64);
            rec_stream.log(
                mesh_anim.get_name().as_str(),
                &Points3D::new(glam_points)
            )?;

            // Send line strip to rerun
            /*MsgSender::new(mesh_anim.get_name().as_str())
                .with_component(&[strip])?
                .with_time(Timeline::new_sequence("frame"), i as i64)
                .send(&rec_stream)
                .unwrap();*/
        }
    }

    // Get bones
    let root_bone = BoneNode::new(&base_milo_loader.obj_dir);

    if let Some(mut root_bone) = root_bone {
        /*root_bone = root_bone
            .children
            .into_iter()
            .find(|b| b.name.eq("bone_pelvis.mesh"))
            .unwrap();*/

        let anim_milo_loader_result = args
            .get(1)
            .map(|p| PathBuf::from(&p))
            .and_then(|p| MiloLoader::from_path(p).ok());

        if let Some(anim_milo_loader) = anim_milo_loader_result {
            let info = &anim_milo_loader.sys_info;
            let anims = anim_milo_loader
                .obj_dir
                .get_entries()
                .iter()
                .filter_map(|o| match o {
                    Object::CharClipSamples(ccs) => Some(ccs),
                    _ => None,
                })
                .collect::<Vec<_>>();

            let char_clip = args
                .get(2)
                .and_then(|anim_name| anims
                    .iter()
                    .find(|a| a
                        .get_name()
                        .eq(anim_name)))
                .unwrap_or_else(|| anims.first().unwrap()); // Default to first one if not found (or_else didn't work...)
            let default_frames = vec![0.0];

            println!("Char clip: {}", char_clip.get_name());

            let bone_samples = [&char_clip.full, &char_clip.one]
                .iter()
                .flat_map(|cbs| cbs
                    .decode_samples(info)
                    .into_iter()
                    .map(|s| (s, if !cbs.frames.is_empty() { &cbs.frames } else { &default_frames })))
                .collect::<Vec<_>>();

            let sample_count = bone_samples
                .iter()
                .map(|(cbs, _)| 0
                    .max(cbs.pos.as_ref().map(|(_, p)| p.len()).unwrap_or_default())
                    .max(cbs.quat.as_ref().map(|(_, q)| q.len()).unwrap_or_default())
                    .max(cbs.rotz.as_ref().map(|(_, r)| r.len()).unwrap_or_default())
                )
                .max()
                .unwrap_or_default();

            let bone_sample_map = bone_samples
                .iter()
                .map(|(cbs, frames)| (cbs.symbol.as_str(), (cbs, *frames)))
                .collect::<HashMap<_, _>>();

            println!("Found {sample_count} samples for {} bones", bone_samples.len());

            /*for (bone, _) in bone_samples.iter() {
                println!("{}", &bone.symbol);
            }*/


            for i in 0..sample_count {
                // If sample not found, use last one?
                // TODO: Iterpolate from frames

                root_bone.recompute_world_anim_transform(na::Matrix4::identity(), &bone_sample_map, i);
                add_bones_to_stream(&root_bone, &rec_stream, i);
            }

            /*for (char_bone_sample, frames) in bone_samples {
                //char_bone_sample.
            }*/
        } else {
            add_bones_to_stream(&root_bone, &rec_stream, 0);

            // Can probably delete
            /*let (points, lines) = generate_bone_points(&root_bone);

            MsgSender::new(root_bone.name)
                .with_component(&points)?
                .send(&rec_stream)
                .unwrap();

            MsgSender::new(format!("{}_lines", root_bone.name))
                .with_component(&lines)?
                .send(&rec_stream)
                .unwrap();*/
        }
    }

    rerun::native_viewer::show(storage.take()).unwrap();

    Ok(())
}

pub struct BoneNode<'a> {
    pub name: &'a str,
    pub object: Option<&'a dyn Trans>,
    pub children: Vec<BoneNode<'a>>,
    pub local_bind_transform: na::Matrix4<f32>,
    pub inverse_bind_transform: na::Matrix4<f32>,
    pub anim_transform: na::Matrix4<f32>,
}

impl<'a> BoneNode<'a> {
    fn new(obj_dir: &'a ObjectDir) -> Option<Self> {
        let dir_name = match obj_dir {
            ObjectDir::ObjectDir(base) => base.name.as_str(),
        };

        // Get bones
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

        // Map children
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

        // Create root node
        let mut root = BoneNode {
            name: dir_name,
            object: None,
            children: Vec::new(),
            local_bind_transform: na::Matrix4::identity(),
            inverse_bind_transform: na::Matrix4::identity().try_inverse().unwrap(),
            anim_transform: na::Matrix4::identity(),
        };

        // Find bones that belong to object dir
        root.children = root.find_child_nodes(root.local_bind_transform, &bones, &child_map);

        if root.children.is_empty() {
            return None;
        }

        Some(root)
    }

    fn find_child_nodes(&self, parent_transform: na::Matrix4<f32>, bone_map: &HashMap<&str, &'a dyn Trans>, child_map: &HashMap<&str, Vec<&'a dyn Trans>>) -> Vec<BoneNode<'a>> {
        let parent_name = self.name;

        let Some(children) = child_map.get(parent_name) else {
            return Vec::new();
        };

        children
            .iter()
            .map(|c| {
                let trans_obj = bone_map.get(c.get_name().as_str()).map(|o| *o);
                let local_transform = trans_obj
                    .map(|o| {
                        let m = o.get_local_xfm();

                        na::Matrix4::new(
                            // Column-major order...
                            m.m11, m.m21, m.m31, m.m41,
                            m.m12, m.m22, m.m32, m.m42,
                            m.m13, m.m23, m.m33, m.m43,
                            m.m14, m.m24, m.m34, m.m44
                        )

                        /*na::Matrix4::new(
                            m.m11, m.m12, m.m13, m.m14,
                            m.m21, m.m22, m.m23, m.m24,
                            m.m31, m.m32, m.m33, m.m34,
                            m.m41, m.m42, m.m43, m.m44
                        )*/
                    })
                    .unwrap_or(na::Matrix4::identity());

                let current_transform = parent_transform * local_transform;

                let mut bone = BoneNode {
                    name: c.get_name().as_str(),
                    object: trans_obj,
                    children: Vec::new(),
                    local_bind_transform: local_transform,
                    inverse_bind_transform: current_transform.try_inverse().unwrap(),
                    anim_transform: current_transform,
                };

                bone.children = bone.find_child_nodes(current_transform, bone_map, child_map);
                bone
            })
            .collect()
    }

    fn recompute_world_anim_transform(&mut self, parent_transform: na::Matrix4<f32>, bone_sample_map: &HashMap<&str, (&CharBoneSample, &Vec<f32>)>, i: usize) {
        let current_transform;

        if let Some((sample, _)) = bone_sample_map.get(self.name) {
            // Decompose original local transform
            let (mut trans, mut rotate, scale) = decompose_trs(self.local_bind_transform);

            // TODO: Multiply by bone weight?
            let pos = sample
                .pos
                .as_ref()
                .and_then(|(_, p)| p.get(i).or_else(|| p.last()))
                .map(|v| na::Vector3::new(v.x, v.y, v.z));
                //.unwrap_or_else(|| na::Vector3::zeros());

            let quat = sample
                .quat
                .as_ref()
                .and_then(|(_, q)| q.get(i).or_else(|| q.last()))
                .map(|q| na::UnitQuaternion::from_quaternion(
                    na::Quaternion::new(
                        q.w,
                        q.x,
                        q.y,
                        q.z
                    )
                ));
                //.unwrap_or_else(|| na::UnitQuaternion::identity());

            let rotz = sample
                .rotz
                .as_ref()
                .and_then(|(_, r)| r.get(i).or_else(|| r.last()))
                .map(|z| na::UnitQuaternion::from_axis_angle(
                    &na::Vector3::z_axis(),
                    std::f32::consts::PI * z
                ));
                //.unwrap_or_else(|| na::UnitQuaternion::identity());

            // Override values if found
            if let Some(pos) = pos {
                trans = pos;
            }

            if let Some(quat) = quat {
                rotate = quat;
            }

            if let Some(rotz) = rotz {
                let (_roll, _pitch, _yaw) = rotate.euler_angles();
                //println!("({}, {}, {})", roll, pitch, yaw);

                //rotate = na::UnitQuaternion::from_euler_angles(roll, pitch, std::f32::consts::PI * rotz);
                //rotate.renormalize();

                /*rotate = na::UnitQuaternion::from_axis_angle(
                    &na::Vector3::z_axis(),
                    std::f32::consts::PI * rotz
                );*/

                rotate *= rotz;
            }

            // Compute local trs transform
            let anim_transform = (na::Matrix4::identity()
                .append_translation(&trans) *
                rotate.to_homogeneous())
                .append_nonuniform_scaling(&scale);


            current_transform = parent_transform * anim_transform;
        } else {
            current_transform = parent_transform * self.local_bind_transform;
        }

        for ch in self.children.iter_mut() {
            ch.recompute_world_anim_transform(current_transform, bone_sample_map, i);
        }

        self.anim_transform = current_transform;
    }

    fn get_world_anim_pos(&self) -> na::Vector3<f32> {
        self.anim_transform.column(3).xyz()
    }
}

fn add_bones_to_stream(bone: &BoneNode, rec_stream: &RecordingStream, i: usize) {
    let v = bone.get_world_anim_pos();

    // Generate line strips
    let strips = bone
        .children
        .iter()
        .map(|c| {
            let cv = c.get_world_anim_pos();
            vec![[v[0], v[1], v[2]], [cv.x, cv.y, cv.z]].into()
        })
        .collect::<Vec<LineStrip3D>>();

    // Add vertex
    /*rec_stream.set_time_sequence("frame", i as i64);
    rec_stream.log_component_batches(
        format!("world/{}", bone.name),
        false,
        [
            &Points3D::new([Position3D::from([v[0], v[1], v[2]])]),
            /*
            .with_splat(Transform3D {
                transform: Transform3DRepr::TranslationRotationScale({
                    let q = na::UnitQuaternion
                        ::from_axis_angle(
                            &na::Vector3::z_axis(),
                            std::f32::consts::PI
                        );

                    TranslationRotationScale3D {
                        rotation: Some(Quaternion::new(q.i, q.j, q.k, q.w).into()),
                        ..Default::default()
                    }
                }),
                from_parent: true
            })
            */
        ]
    );*/

    // Add lines from node to children
    rec_stream.set_time_sequence("frame", i as i64);
    rec_stream.log(
        format!("world/{}/lines", bone.name),
        &rerun::LineStrips3D::new(strips)
    ).unwrap();

    // Add direction arrow (not working)
    /*MsgSender::new(format!("world/{}/arrows", bone.name))
        .with_component(&[
            Arrow3D {
                origin: Vec3D([v[0], v[1], v[2]]),
                vector: {
                    let v: na::Vector3<_> = bone
                        .anim_transform
                        .transform_vector(&na::Vector3::from_element(0.5));

                    /*let rotation = na::UnitQuaternion::from_matrix(&bone
                        .world_bind_transform.fixed_view::<3, 3>(0, 0).into()
                    );

                    let (i, j, k) = rotation.euler_angles();
                    let v = na::Vector3::new(i, j, k);*/

                    Vec3D([v[0], v[1], v[2]])
                }
            }
        ])
        .unwrap()
        //.with_splat(Scalar(10.)).unwrap()
        .with_splat(Radius(0.05)).unwrap()
        .with_splat(ColorRGBA::from_rgb(0, 255, 0)).unwrap()
        /*.with_splat(ViewCoordinates::from_up_and_handedness(
            SignedAxis3::POSITIVE_Z,
            Handedness::Right))
        .unwrap()*/
        .with_time(Timeline::new_sequence("frame"), i as i64)
        .send(rec_stream)
        .unwrap();*/

    for ch in bone.children.iter() {
        add_bones_to_stream(ch, rec_stream, i);
    }
}

// TODO: Move to shared part in lib
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