mod io;

use std::collections::HashMap;

use crate::{scene::{Quat, Vector3}, SystemInfo, io::IOEndian};
use grim_macros::*;
use grim_traits::scene::*;
pub use io::*;

const MAX_ROTATION_DEGREES: f32 = 1080.0; // 3 rotations

#[derive(Debug, Clone, Default)]
pub struct CharBone4Bone {
    pub symbol: String, // Bone name + transform property ext
    pub weight: f32,
}

#[derive(Debug, Default)]
pub struct CharBoneSample {
    pub symbol: String, // Bone name
    pub pos: Option<(f32, Vec<Vector3>)>,
    pub quat: Option<(f32, Vec<Quat>)>,
    pub rotz: Option<(f32, Vec<f32>)>,
}

#[derive(Debug)]
pub enum EncodedSamples {
    Compressed(Vec<CharBone4Bone>, Vec<Box<[u8]>>), // Raw sample collection of bone transforms
    Uncompressed(Vec<CharBoneSample>) // Collections of samples grouped by bone transforms
}

impl Default for EncodedSamples {
    fn default() -> Self {
        EncodedSamples::Uncompressed(Vec::new())
    }
}

#[derive(Debug, Default)]
pub struct CharBonesSamples { // Sample set
    pub bones: Vec<CharBone4Bone>,
    pub compression: u32, // TODO: Convert to enum?
    pub counts: [u32; 7], // Offsets
    pub computed_sizes: [u32; 7],
    pub computed_flags: u32,

    pub samples: EncodedSamples,
    pub frames: Vec<f32>
}

impl CharBonesSamples {
    pub fn get_type_of<T: AsRef<str>>(name: T) -> u32 {
        let name = name.as_ref();

        let last_dot_idx = name.find('.');
        if last_dot_idx.is_none() {
            return 7;
        }

        let ext = name[last_dot_idx.unwrap()..].to_ascii_lowercase();
        match ext.as_str() {
            ".pos" => 0,
            ".scale" => 1,
            ".quat" => 2,
            ".rotx" => 3,
            ".roty" => 4,
            ".rotz" => 5,
            _ => 7,
        }
    }

    pub fn get_type_size(&self, idx: u32) -> u32 {
        if idx < 2 {
            return if self.compression < 2 { 16 } else { 6 };
        }

        if idx != 2 {
            return if self.compression == 0 { 4 } else { 2 };
        }

        if self.compression > 2 {
            return 4;
        }

        if self.compression == 0 {
            return 16;
        }

        return 8;
    }

    pub(crate) fn get_type_size2(&self, idx: u32) -> usize {
        // Note: Not sure if scale ever gets compressed
        const SIZES: [[usize; 6]; 4] = [
        //    p  s   q  x  y  z
            [12, 4, 16, 4, 4, 4], // 0 Uncompressed
            [12, 4,  8, 2, 2, 2], // 1 Compress rots
            [ 6, 4,  8, 2, 2, 2], // 2 Compress vects
            [ 6, 4,  4, 2, 2, 2], // 3 Compress quats
        ];

        SIZES
            .get(self.compression as usize)
            .and_then(|r| r.get(idx as usize))
            .map(|s| *s)
            .unwrap_or_default()
    }

    pub fn recompute_sizes(&mut self) {
        self.computed_sizes[0] = 0;

        for i in 0..6 {
            // Next count bleeds into computed sizes
            // In C++ code, the same array was likely shared
            let curr_count = self.counts[i];
            let next_count = self.counts[i + 1];

            let type_size = self.get_type_size(i as u32);

            self.computed_sizes[i + 1] = self.computed_sizes[i] + (next_count - curr_count) * type_size;
        }

        self.computed_flags = (self.computed_sizes.last().unwrap() + 0xF) & 0xFFFF_FFF0;
    }

    pub fn generate_bones_from_samples(&mut self) {
        self.bones = match &self.samples {
            EncodedSamples::Compressed(bones, _) => bones.to_vec(),
            EncodedSamples::Uncompressed(samples) => {
                let (pos, quat, rotz) = samples
                    .iter()
                    .fold((Vec::new(), Vec::new(), Vec::new()), |(mut pos, mut quat, mut rotz), s| {
                        if let Some((w, _)) = s.pos {
                            pos.push(CharBone4Bone {
                                symbol: format!("{}.pos", s.symbol),
                                weight: w,
                            });
                        }

                        if let Some((w, _)) = s.quat {
                            quat.push(CharBone4Bone {
                                symbol: format!("{}.quat", s.symbol),
                                weight: w,
                            });
                        }

                        if let Some((w, _)) = s.rotz {
                            rotz.push(CharBone4Bone {
                                symbol: format!("{}.rotz", s.symbol),
                                weight: w,
                            });
                        }

                        (pos, quat, rotz)
                    });

                pos
                    .into_iter()
                    .chain(quat)
                    .chain(rotz)
                    .collect()
            },
        };
    }

    pub fn decode_samples(&self, sys_info: &SystemInfo) -> Vec<CharBoneSample> {
        let EncodedSamples::Compressed(bones, compressed_samples) = &self.samples else {
            // Maybe throw error?
            return Vec::new();
        };

        let read_f32 = if sys_info.endian.eq(&IOEndian::Big) {
            |data: [u8; 4]| -> f32 { f32::from_be_bytes(data) }
        } else {
            |data: [u8; 4]| -> f32 { f32::from_le_bytes(data) }
        };

        let read_packed_f32 = if sys_info.endian.eq(&IOEndian::Big) {
            |data: [u8; 2]| -> f32 {
                ((i16::from_be_bytes(data) as f32) / 32767.0).max(-1.0)
            }
        } else {
            |data: [u8; 2]| -> f32 {
                ((i16::from_le_bytes(data) as f32) / 32767.0).max(-1.0)
            }
        };

        // Group by bone name
        let mut bone_map = HashMap::new();
        for sample in compressed_samples {
            let mut i = 0usize;

            for bone in bones {
                // Compute bone name
                // TODO: Make more efficient
                let bone_name = bone.symbol.to_owned()
                    .replace(".pos", "")
                    .replace(".quat", "")
                    .replace(".rotz", "");

                // Get or insert bone sample w/ name
                let bone_sample = bone_map
                    .entry(bone_name.to_owned())
                    .or_insert_with(|| CharBoneSample {
                        symbol: bone_name,
                        ..Default::default()
                    });

                match Self::get_type_of(bone.symbol.as_str()) {
                    t @ 0 => {
                        // pos
                        let pos = match self.get_type_size2(t) {
                            s @ 12 => {
                                // Read data
                                let x = read_f32([sample[i    ], sample[i + 1], sample[i + 2], sample[i + 3]]);
                                let y = read_f32([sample[i + 4], sample[i + 5], sample[i + 6], sample[i + 7]]);
                                let z = read_f32([sample[i + 8], sample[i + 9], sample[i + 10], sample[i + 11]]);

                                i += s as usize;
                                Vector3 { x, y, z }
                            },
                            s @ 6 => {
                                // Read packed data
                                let x = read_packed_f32([sample[i    ], sample[i + 1]]) * 1345.; // TODO: Investigate better constant
                                let y = read_packed_f32([sample[i + 2], sample[i + 3]]) * 1345.;
                                let z = read_packed_f32([sample[i + 4], sample[i + 5]]) * 1345.;

                                i += s as usize;
                                Vector3 { x, y, z }
                            },
                            s @ _ => panic!("Unsupported .pos compression of type {}", s)
                        };

                        // Insert or append pos sample
                        bone_sample.pos = match bone_sample.pos.take() {
                            Some((w, mut samples)) => {
                                samples.push(pos);
                                Some((w, samples))
                            },
                            _ => Some((bone.weight, vec![pos]))
                        }
                    },
                    t @ 2 => {
                        // quat
                        let quat = match self.get_type_size2(t) {
                            s @ 16 => {
                                // Read data
                                let x = read_f32([sample[i    ], sample[i + 1], sample[i + 2], sample[i + 3]]);
                                let y = read_f32([sample[i + 4], sample[i + 5], sample[i + 6], sample[i + 7]]);
                                let z = read_f32([sample[i + 8], sample[i + 9], sample[i + 10], sample[i + 11]]);
                                let w = read_f32([sample[i + 12], sample[i + 13], sample[i + 14], sample[i + 15]]);

                                i += s as usize;
                                Quat { x, y, z, w }
                            },
                            s @ 8 => {
                                // Read packed data
                                let x = read_packed_f32([sample[i    ], sample[i + 1]]);
                                let y = read_packed_f32([sample[i + 2], sample[i + 3]]);
                                let z = read_packed_f32([sample[i + 4], sample[i + 5]]);
                                let w = read_packed_f32([sample[i + 6], sample[i + 7]]);

                                i += s as usize;
                                Quat { x, y, z, w }
                            },
                            s @ _ => panic!("Unsupported .pos compression of type {}", s)
                        };

                        // Insert or append quat sample
                        bone_sample.quat = match bone_sample.quat.take() {
                            Some((w, mut samples)) => {
                                samples.push(quat);
                                Some((w, samples))
                            },
                            _ => Some((bone.weight, vec![quat]))
                        }
                    },
                    t @ 5 => {
                        // rotz
                        let rotz = match self.get_type_size2(t) {
                            s @ 4 => {
                                // Read data
                                let x = read_f32([sample[i    ], sample[i + 1], sample[i + 2], sample[i + 3]]);

                                i += s as usize;
                                x
                            },
                            s @ 2 => {
                                // Read packed data
                                let x = read_packed_f32([sample[i    ], sample[i + 1]]) * MAX_ROTATION_DEGREES;

                                i += s as usize;
                                x
                            },
                            s @ _ => panic!("Unsupported .pos compression of type {}", s)
                        };

                        // Insert or append rotz sample
                        bone_sample.rotz = match bone_sample.rotz.take() {
                            Some((w, mut samples)) => {
                                samples.push(rotz);
                                Some((w, samples))
                            },
                            _ => Some((bone.weight, vec![rotz]))
                        }
                    },
                    t @ _ => panic!("Unsupported bone transform of type {t}"),
                };
            }

            // Interpret sample data...
        }

        let mut bone_samples = bone_map.into_values().collect::<Vec<_>>();

        /*println!("Found {} bones", bones.len());
        for bone in bone_samples.iter() {
            println!("\t{} ({} pos) ({} quat) ({} rotz)",
                &bone.symbol,
                bone.pos.as_ref().map(|(_, p)| p.len()).unwrap_or_default(),
                bone.quat.as_ref().map(|(_, q)| q.len()).unwrap_or_default(),
                bone.rotz.as_ref().map(|(_, r)| r.len()).unwrap_or_default()
            );
        }*/

        /*for bone in bone_samples.iter() {
            let Some((_, rotz)) = bone.rotz.as_ref() else {
                continue;
            };

            let (min, max, sum) = rotz
                .iter()
                .fold(
                    (1.0f32, -1.0f32, 0.0f32),
                    |(min, max, sum), rotz| (min.min(*rotz), max.max(*rotz), sum + *rotz)
                );

            let avg = sum / rotz.len() as f32;

            println!("{}: min = {}, max = {}, avg = {}",
                &bone.symbol,
                min,
                max,
                avg
            );
        }*/

        // Sort by name
        bone_samples.sort_by(|a, b| a.symbol.cmp(&b.symbol));

        bone_samples
    }

    pub(crate) fn get_sample_count(&self) -> usize {
        match &self.samples {
            EncodedSamples::Compressed(_, raw_samples) => raw_samples.len(),
            EncodedSamples::Uncompressed(samples) => samples
                .iter()
                .fold(0, |acc, s| {
                    acc
                        .max(s.pos.as_ref().map(|(_, p)| p.len()).unwrap_or_default())
                        .max(s.quat.as_ref().map(|(_, q)| q.len()).unwrap_or_default())
                        .max(s.rotz.as_ref().map(|(_, r)| r.len()).unwrap_or_default())
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use super::*;

    #[rstest]
    #[case("", 7)]
    #[case("bone", 7)]
    #[case("bone.", 7)]
    #[case("bone.pdf", 7)]
    #[case("bone.pos", 0)]
    #[case("bone.scale", 1)]
    #[case("bone.quat", 2)]
    #[case("bone.rotx", 3)]
    #[case("bone.roty", 4)]
    #[case("bone.rotz", 5)]
    fn char_bones_get_type_of(#[case] input_name: &str, #[case] expected: u32) {
        assert_eq!(expected, CharBonesSamples::get_type_of(input_name));
    }

    #[rstest]
    #[case(0, 0, 16)]
    #[case(0, 1, 16)]
    #[case(0, 2, 16)]
    #[case(0, 3, 4)]
    #[case(0, 4, 4)]
    #[case(0, 5, 4)]
    #[case(0, 6, 4)]
    #[case(1, 0, 16)]
    #[case(1, 1, 16)]
    #[case(1, 2, 8)]
    #[case(1, 3, 2)]
    #[case(1, 4, 2)]
    #[case(1, 5, 2)]
    #[case(1, 6, 2)]
    #[case(2, 0, 6)]
    #[case(2, 1, 6)]
    #[case(2, 2, 8)]
    #[case(2, 3, 2)]
    #[case(2, 4, 2)]
    #[case(2, 5, 2)]
    #[case(2, 6, 2)]
    fn char_bones_get_type_size(#[case] input_compression: u32, #[case] input_idx: u32, #[case] expected: u32) {
        let char_bone = CharBonesSamples {
            compression: input_compression,
            ..Default::default()
        };

        let result = char_bone.get_type_size(input_idx);
        assert_eq!(expected, result);
    }

    #[rstest]
    #[case(1, [0, 1, 1, 22, 22, 22, 32], [0, 16, 16, 184, 184, 184, 204], 208)]
    #[case(2, [0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0], 0)]
    //#[case(2, [0, 27, 27, 37, 37, 37, 37], [0, 216, 216, 352, 352, 352, 352], 352)]
    #[case(2, [0, 36, 36, 53, 53, 53, 53], [0, 216, 216, 352, 352, 352, 352], 352)]
    fn char_bones_recompute_sizes(#[case] input_compression: u32, #[case] input_counts: [u32; 7], #[case] expected_computed_sizes: [u32; 7], #[case] expected_computed_flags: u32) {
        let mut char_bone = CharBonesSamples {
            compression: input_compression,
            counts: input_counts,
            ..Default::default()
        };

        char_bone.recompute_sizes();

        assert_eq!(expected_computed_sizes, char_bone.computed_sizes);
        assert_eq!(expected_computed_flags, char_bone.computed_flags);
    }
}