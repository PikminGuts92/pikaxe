use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use grim_traits::scene::*;
use thiserror::Error as ThisError;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, ThisError)]
pub enum CharBonesSamplesReadError {
    #[error("CharBonesSamples version of {version} not supported")]
    CharBonesSamplesNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
        16 => true, // TBRB/GDRB
         _ => false
    }
}

fn compute_counts(char_bone_samples: &CharBonesSamples) -> [u32; 7] {
    let mut counts = [0u32; 7];

    let trans_counts = char_bone_samples
        .bones
        .iter()
        .fold([0u32; 7], |mut sizes, b| {
            let idx = CharBonesSamples::get_type_of(&b.symbol);
            sizes[idx as usize] += 1;
            sizes
        });

    let mut current_count = 0;
    for i in 0..counts.len() {
        counts[i] = current_count;
        current_count += trans_counts[i];
    }

    counts
}

pub(crate) fn load_char_bones_samples(char_bones_samples: &mut CharBonesSamples, reader: &mut Box<BinaryStream>, _info: &SystemInfo) -> Result<(), Box<dyn Error>> {
    let version = reader.read_uint32()?;

    // If not valid, return unsupported error
    if !is_version_supported(version) {
        return Err(Box::new(CharBonesSamplesReadError::CharBonesSamplesNotSupported {
            version
        }));
    }

    let (bones, sample_count) = load_char_bones_samples_header(char_bones_samples, reader, version)?;
    load_char_bones_samples_data(char_bones_samples, reader, version, bones, sample_count)?;

    Ok(())
}

pub(crate) fn load_char_bones_samples_header(char_bones_samples: &mut CharBonesSamples, reader: &mut Box<BinaryStream>, version: u32) -> Result<(Vec<(String, f32)>, u32), Box<dyn Error>> {
    // Earlier versions use 10 counts. Though the extra values can be ignored.
    let count_size = if version > 15 { 7 } else { 10 };

    let bone_count = reader.read_uint32()?;
    let mut bones = Vec::new();

    for _ in 0..bone_count {
        // Read symbol + weight values
        // Note: Pre-RB games don't use weighted bones so default to 1.0
        let name = reader.read_prefixed_string()?;
        let weight = if version <= 10 { 1.0 } else { reader.read_float32()? };

        bones.push((name, weight));
    }

    char_bones_samples.bones = bones
        .iter()
        .map(|(name, weight)| CharBone {
            symbol: name.to_owned(),
            weight: *weight,
        })
        .collect();

    // Read offset values
    for i in 0..count_size {
        let v = reader.read_uint32()?;

        // Only add if there's room
        if let Some(c) = char_bones_samples.counts.get_mut(i) {
            *c = v;
        }
    }

    char_bones_samples.compression = reader.read_uint32()?;
    let sample_count = reader.read_uint32()?;

    // Read frames
    // Pre-RB games don't use interpolated frames?
    char_bones_samples.frames = match version {
        v @ _ if v > 11 => {
            // Read frames
            let frame_count = reader.read_uint32()?;
            let mut frames = Vec::new();

            for _ in 0..frame_count {
                let frame = reader.read_float32()?;
                frames.push(frame);
            }

            frames
        },
        _ => Vec::new()
    };

    Ok((bones, sample_count))
}

pub(crate) fn load_char_bones_samples_data(char_bones_samples: &mut CharBonesSamples, reader: &mut Box<BinaryStream>, version: u32, bones: Vec<(String, f32)>, sample_count: u32) -> Result<(), Box<dyn Error>> {
    /*if let Some(v) && v == 4 {

    }*/

    // Calculate sample frame size
    // TODO: Support 10 counts
    /*let mut size = 0;
    let mut curr_num = 0;

    for i in 0..char_bones_samples.counts.len() {
        let type_size = char_bones_samples.get_type_size(i as u32);
    }*/

    // Calculate stride
    let mut sample_size: usize = bones
        .iter()
        .filter_map(|(s, _)| match CharBonesSamples::get_type_of(s) {
            i @ 0..=6 => Some(char_bones_samples.get_type_size2(i)),
            _ => None
        })
        .sum();

    // Seems that only RB-era samples are byte aligned...
    if version > 11 {
        sample_size = crate::io::align_to_multiple_of_four(sample_size);
    }

    // Read samples
    let mut samples = Vec::new();
    for _ in 0..sample_count {
        let data = reader.read_bytes(sample_size)?;
        samples.push(data.into_boxed_slice());
    }

    char_bones_samples.samples = EncodedSamples::Compressed(
        bones.into_iter().map(|(s, w)| CharBone { symbol: s, weight: w }).collect(),
        samples
    );

    Ok(())
}

pub(crate) fn save_char_bones_samples_header(char_bones_samples: &CharBonesSamples, writer: &mut Box<BinaryStream>, version: u32) -> Result<(), Box<dyn Error>> {
    // Earlier versions use 10 counts. Though the extra values can be ignored.
    let count_size = if version > 15 { 7 } else { 10 };

    // Write bones
    writer.write_uint32(char_bones_samples.bones.len() as u32)?;
    for bone in char_bones_samples.bones.iter() {
        // Read symbol + weight values
        // Note: Pre-RB games don't use weighted bones so skip for PS2 GH2

        writer.write_prefixed_string(&bone.symbol)?;
        if version >= 11 {
            writer.write_float32(bone.weight)?;
        }
    }

    // Write offset values
    let counts = compute_counts(char_bones_samples);
    for i in 0..count_size {
        let count_value = counts
            .get(i)
            .map(|o| *o)
            .unwrap_or_else(|| counts.last().map(|o| *o).unwrap());

        writer.write_uint32(count_value)?;
    }

    writer.write_uint32(char_bones_samples.compression)?;
    writer.write_uint32(char_bones_samples.get_sample_count() as u32)?;

    // No frames for v11

    Ok(())
}

pub(crate) fn save_char_bones_samples_data(char_bones_samples: &CharBonesSamples, writer: &mut Box<BinaryStream>, version: u32) -> Result<(), Box<dyn Error>> {
    let samples = match &char_bones_samples.samples {
        EncodedSamples::Uncompressed(samples) => samples,
        EncodedSamples::Compressed(_, raw_samples) => {
            for raw_sample in raw_samples {
                writer.write_bytes(raw_sample)?;
            }
            return Ok(());
        }
    };

    let empty_vector3 = Vector3::default();
    let empty_quat = Quat::default();

    let write_vector = if char_bones_samples.compression < 2 {
        save_vector3
    } else {
        save_vector3_packed
    };

    let write_quat = if char_bones_samples.compression == 0 {
        save_quat
    } else {
        save_quat_packed
    };

    let write_rot = if char_bones_samples.compression == 0 {
        save_rot
    } else {
        save_rot_packed
    };

    let (sample_count, pos_samples, quat_samples, rotz_samples) = samples
        .iter()
        .fold((0, Vec::new(), Vec::new(), Vec::new()), |(mut sample_count, mut pos_samples, mut quat_samples, mut rotz_samples), s| {
            if let Some((_, p)) = &s.pos {
                sample_count = sample_count.max(p.len());
                pos_samples.push(p);
            }
            if let Some((_, q)) = &s.quat {
                sample_count = sample_count.max(q.len());
                quat_samples.push(q);
            }
            if let Some((_, rz)) = &s.rotz {
                sample_count = sample_count.max(rz.len());
                rotz_samples.push(rz);
            }

            (sample_count, pos_samples, quat_samples, rotz_samples)
        });

    for i in 0..sample_count {
        // Write positions
        for pos in pos_samples.iter() {
            let sample = pos
                .get(i)
                .or_else(|| pos.last());

            if let Some(sample) = sample {
                write_vector(sample, writer)?;
            } else {
                write_vector(&empty_vector3, writer)?;
            }
        }

        // Write quaternions
        for quat in quat_samples.iter() {
            let sample = quat
                .get(i)
                .or_else(|| quat.last());

            if let Some(sample) = sample {
                write_quat(sample, writer)?;
            } else {
                write_quat(&empty_quat, writer)?;
            }
        }

        // Write z-rotations
        for rotz in rotz_samples.iter() {
            let sample = rotz
                .get(i)
                .or_else(|| rotz.last());

            if let Some(sample) = sample {
                write_rot(*sample, writer)?;
            } else {
                write_rot(0.0, writer)?;
            }
        }
    }

    Ok(())
}

fn save_rot(value: f32, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    writer.write_float32(value)
}

fn save_rot_packed(value: f32, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    // Convert to signed short and write
    let s = ((value / 1080.) * 32767.0).round() as i16;
    writer.write_int16(s)
}