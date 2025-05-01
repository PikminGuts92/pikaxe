use crate::Platform;
use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use grim_traits::scene::*;
use std::error::Error;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum MeshLoadError {
    #[error("Mesh version {version} is not supported")]
    MeshVersionNotSupported {
        version: u32
    },
    #[error("BSP volume geometry is not supported")]
    BSPVolumeNotSupported,
}

fn is_version_supported(version: u32) -> bool {
    match version {
        10 => true,      // Freq
        13 | 14 => true, // Amp Demo/Amp
        22 => true,      // AntiGrav
        25 => true,      // GH1
        28 => true,      // GH2/GH2 360
        33 => true,
        34 => true,      // RB1/RB2
        36 | 37 => true, // TBRB/GDRB
        38 => true,      // RB3
        _ => false
    }
}

impl ObjectReadWrite for MeshObject {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;
        if !is_version_supported(version) {
            return Err(Box::new(MeshLoadError::MeshVersionNotSupported {
                version
            }));
        }

        // Read as null-terminated or prefixed string depending on version
        let read_string = if version <= 10 {
            |reader: &mut Box<BinaryStream>| reader.read_null_terminated_string()
        } else {
            |reader: &mut Box<BinaryStream>| reader.read_prefixed_string()
        };

        load_object(self, &mut reader, info)?;
        load_trans(self, &mut reader, info, false)?;
        load_draw(self, &mut reader, info, false)?;

        if version < 15 {
            _ = reader.seek(SeekFrom::Current(4)); // Always 0?

            // Read bones
            self.bones.clear();
            let bone_count = reader.read_uint32()?;

            for _ in 0..bone_count {
                let mut bone = BoneTrans::default();

                bone.name = read_string(&mut reader)?;
                self.bones.push(bone);
            }
        }

        if version < 20 {
            // Skip 2 ints
            reader.seek(SeekFrom::Current(8))?;
        }

        if version < 3 {
            // Skip int
            reader.seek(SeekFrom::Current(4))?;
        }

        self.mat = read_string(&mut reader)?;
        if version == 27 {
            // Secondary material?
            _ = reader.read_prefixed_string()?;
        }

        self.geom_owner = read_string(&mut reader)?;
        if version < 13 {
            // Secondary geom owner?
            _ = read_string(&mut reader)?;
        }

        if version < 15 {
            // Set on mesh instead of base trans object
            let trans_parent = read_string(&mut reader)?;
            self.set_parent(trans_parent);
        }

        if version < 14 {
            // Skip empty RndTransformable strings
            _ = read_string(&mut reader)?;
            _ = read_string(&mut reader)?;
        }

        if version < 3 {
            // Skip vector3
            reader.seek(SeekFrom::Current(12))?;
        }

        if version < 15 {
            // Set on mesh instead of base draw object
            load_sphere(self.get_sphere_mut(), &mut reader)?;
        }

        if version < 8 {
            // Skip some bool
            _ = reader.read_boolean()?;
        }

        if version < 15 {
            // Skip unknown string + float
            _ = read_string(&mut reader)?;
            reader.seek(SeekFrom::Current(4))?;
        }

        // Read mutable value
        if version < 16 {
            if version > 11 {
                _ = reader.read_boolean()?;
            }

            // Just hard-code as default value
            self.mutable = Mutable::kMutableNone;
        } else {
            self.mutable = reader.read_uint32()?.into();
        }

        // Read volume value
        if version > 17 {
            self.volume = reader.read_uint32()?.into();
        }

        // Read bsp value
        if version > 18 {
            // Ignored but still validated
            let bsp = reader.read_uint8()?;
            if bsp != 0 {
                //panic!("Expected bsp field to be 0, not \"{}\" in Mesh", bsp);
                return Err(Box::new(MeshLoadError::BSPVolumeNotSupported));
            }
        }

        if version == 7 {
            // Skip another unknown bool
            _ = reader.read_boolean()?;
        }

        if version < 11 {
            // Skip unknown int
            reader.seek(SeekFrom::Current(4))?;
        }

        let vert_count = reader.read_uint32()?;
        let mut is_ng = false;

        if version >= 36 {
            is_ng = reader.read_boolean()?;

            // If next gen, read stride + 1 constant
            if is_ng {
                reader.seek(SeekFrom::Current(8))?; // (36, 1) | (40, 2)
            }
        }

        self.vertices.clear();
        self.raw_vertices.clear();
        for _ in 0..vert_count {
            let mut vec = Vert::default();

            // TODO: Should probably clean up this loop
            if version <= 10 {
                // Freq (56 bytes)
                // Position
                vec.pos.x = reader.read_float32()?;
                vec.pos.y = reader.read_float32()?;
                vec.pos.z = reader.read_float32()?;

                // Normals
                vec.normals.x = reader.read_float32()?;
                vec.normals.y = reader.read_float32()?;
                vec.normals.z = reader.read_float32()?;

                // UVs
                vec.uv.u = reader.read_float32()?;
                vec.uv.v = reader.read_float32()?;

                // Weights
                vec.weights[0] = reader.read_float32()?;
                vec.weights[1] = reader.read_float32()?;
                vec.weights[2] = reader.read_float32()?;
                vec.weights[3] = reader.read_float32()?;

                // Bone indices
                vec.bones[0] = reader.read_uint16()?;
                vec.bones[1] = reader.read_uint16()?;
                vec.bones[2] = reader.read_uint16()?;
                vec.bones[3] = reader.read_uint16()?;

                self.vertices.push(vec);
                continue;
            } else if version <= 22 {
                // Amp/AntiGrav (56 bytes)
                // Position
                vec.pos.x = reader.read_float32()?;
                vec.pos.y = reader.read_float32()?;
                vec.pos.z = reader.read_float32()?;

                // Bone indices
                vec.bones[0] = reader.read_uint16()?;
                vec.bones[1] = reader.read_uint16()?;
                vec.bones[2] = reader.read_uint16()?;
                vec.bones[3] = reader.read_uint16()?;

                // Normals
                vec.normals.x = reader.read_float32()?;
                vec.normals.y = reader.read_float32()?;
                vec.normals.z = reader.read_float32()?;

                // Weights
                vec.weights[0] = reader.read_float32()?;
                vec.weights[1] = reader.read_float32()?;
                vec.weights[2] = reader.read_float32()?;
                vec.weights[3] = reader.read_float32()?;

                // UVs
                vec.uv.u = reader.read_float32()?;
                vec.uv.v = reader.read_float32()?;

                self.vertices.push(vec);
                continue;
            }

            // TODO: Remove once next gen vertex format is figured out
            if version >= 36 && is_ng {
                // Read raw vert data
                let mut raw_vert = [0u8; 36];
                let data = reader.read_bytes(36)?;
                raw_vert.copy_from_slice(data.as_slice());

                reader.seek(SeekFrom::Current(-36))?;
                self.raw_vertices.push(raw_vert);
            }

            // Position
            vec.pos.x = reader.read_float32()?;
            vec.pos.y = reader.read_float32()?;
            vec.pos.z = reader.read_float32()?;
            if version == 34 {
                vec.pos.w = reader.read_float32()?;
            }

            if version < 35 || !is_ng {
                if version >= 38 {
                    // Skip extra bytes
                    // TODO: Figure out what this data is...
                    reader.seek(SeekFrom::Current(16))?;
                }

                // Normals
                vec.normals.x = reader.read_float32()?;
                vec.normals.y = reader.read_float32()?;
                vec.normals.z = reader.read_float32()?;
                if version == 34 {
                    vec.normals.w = reader.read_float32()?;
                }

                if version >= 38 {
                    // Packed in different order?
                    // UVs
                    vec.uv.u = reader.read_float32()?;
                    vec.uv.v = reader.read_float32()?;

                    // Weights
                    vec.weights[0] = reader.read_float32()?;
                    vec.weights[1] = reader.read_float32()?;
                    vec.weights[2] = reader.read_float32()?;
                    vec.weights[3] = reader.read_float32()?;
                } else {
                    // Weights
                    vec.weights[0] = reader.read_float32()?;
                    vec.weights[1] = reader.read_float32()?;
                    vec.weights[2] = reader.read_float32()?;
                    vec.weights[3] = reader.read_float32()?;

                    // UVs
                    vec.uv.u = reader.read_float32()?;
                    vec.uv.v = reader.read_float32()?;
                }

                if version >= 33 {
                    // Bone indices
                    vec.bones[0] = reader.read_uint16()?;
                    vec.bones[1] = reader.read_uint16()?;
                    vec.bones[2] = reader.read_uint16()?;
                    vec.bones[3] = reader.read_uint16()?;

                    if version >= 38 {
                        // Skip unknown bytes
                        // TODO: Figure out what this data is...
                        reader.seek(SeekFrom::Current(16))?;
                    } else {
                        // Tangent?
                        vec.tangent.x = reader.read_float32()?;
                        vec.tangent.y = reader.read_float32()?;
                        vec.tangent.z = reader.read_float32()?;
                        vec.tangent.w = reader.read_float32()?;
                    }
                }
            } else {
                let uv_check = reader.read_int32()?;

                if uv_check == -1 {
                    // UVs
                    vec.uv.u = reader.read_float16()?.into();
                    vec.uv.v = reader.read_float16()?.into();

                    // Normals
                    vec.normals.x = reader.read_float16()?.into();
                    vec.normals.y = reader.read_float16()?.into();
                    vec.normals.z = reader.read_float16()?.into();

                    // Not sure
                    reader.seek(SeekFrom::Current(6))?;

                    // Bone indices
                    vec.bones[0] = reader.read_uint8()?.into();
                    vec.bones[1] = reader.read_uint8()?.into();
                    vec.bones[2] = reader.read_uint8()?.into();
                    vec.bones[3] = reader.read_uint8()?.into();
                } else {
                    // Read as regular uvs
                    reader.seek(SeekFrom::Current(-4))?;

                    // UVs
                    vec.uv.u = reader.read_float16()?.into();
                    vec.uv.v = reader.read_float16()?.into();

                    // Normals
                    vec.normals.x = reader.read_float16()?.into();
                    vec.normals.y = reader.read_float16()?.into();
                    vec.normals.z = reader.read_float16()?.into();
                    reader.seek(SeekFrom::Current(2))?; // Not sure

                    // Not sure
                    reader.seek(SeekFrom::Current(4))?;

                    // Bone indices
                    vec.bones[0] = reader.read_uint16()?;
                    vec.bones[1] = reader.read_uint16()?;
                    vec.bones[2] = reader.read_uint16()?;
                    vec.bones[3] = reader.read_uint16()?;
                }

                if version >= 38 {
                    // Skip extra bytes
                    // TODO: Figure out what this data is...
                    reader.seek(SeekFrom::Current(4))?;
                }
            }

            self.vertices.push(vec);
        }

        let face_count = reader.read_uint32()?;
        self.faces.clear();
        for _ in 0..face_count {
            let mut face = [0u16; 3];

            face[0] = reader.read_uint16()?;
            face[1] = reader.read_uint16()?;
            face[2] = reader.read_uint16()?;

            self.faces.push(face);
        }

        if version < 24 {
            // Read pairs of shorts, matches face count
            // Not always present, skip for now
            let short_count = reader.read_uint32()?;
            reader.seek(SeekFrom::Current((short_count * 2) as i64 * std::mem::size_of::<u16>() as i64))?;

            if version >= 22 {
                // TODO: Preserve this data
                let group_count = reader.read_uint32()?;

                for _ in 0..group_count {
                    reader.seek(SeekFrom::Current(4))?; // Some number

                    let short_count = reader.read_uint32()?;
                    reader.seek(SeekFrom::Current(short_count as i64 * 2))?;

                    let int_count = reader.read_uint32()?;
                    reader.seek(SeekFrom::Current(int_count as i64 * 4))?;
                }
            }

            if version >= 14 {
                // Skip int
                reader.seek(SeekFrom::Current(4))?;
            }

            return Ok(());
        }

        let group_count = reader.read_uint32()?;
        self.face_groups = reader.read_bytes(group_count as usize)?;

        // Read bones
        self.bones.clear();
        if version >= 34 {
            let bone_count = reader.read_uint32()?;

            for _ in 0..bone_count {
                let mut bone = BoneTrans::default();

                bone.name = reader.read_prefixed_string()?;
                load_matrix(&mut bone.trans, &mut reader)?;

                self.bones.push(bone);
            }
        } else {
            // Should be 0 or 4 bones
            let mut bones = Vec::new();

            // Read bone names
            for i in 0..4 {
                let name = reader.read_prefixed_string()?;
                if i == 0 && name.is_empty() {
                    break;
                }

                bones.push(BoneTrans {
                    name,
                    ..Default::default()
                });
            }

            // Read bone transforms
            for mut bone in bones {
                load_matrix(&mut bone.trans, &mut reader)?;

                // Add bone if it has name
                if !bone.name.is_empty() {
                    self.bones.push(bone);
                }
            }
        }

        if version >= 36 {
            self.keep_mesh_data = reader.read_boolean()?;
        }

        if version == 37 {
            self.exclude_from_self_shadow = reader.read_boolean()?;
        } else if version >= 38 {
            self.has_ao_calculation = reader.read_boolean()?;
        }

        // TODO: Parse extra data from previous gen platforms

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 34;
        let is_ng = info.is_next_gen();

        stream.write_uint32(version)?;

        save_object(self, &mut stream, info)?;
        save_trans(self, &mut stream, info, false)?;
        save_draw(self, &mut stream, info, false)?;

        stream.write_prefixed_string(&self.mat)?;
        stream.write_prefixed_string(&self.geom_owner)?;

        stream.write_uint32(self.mutable as u32)?;
        stream.write_uint32(self.volume as u32)?;

        // TODO: Figure out what bsp? Although it's not really used at all...
        stream.write_uint8(0)?;

        stream.write_uint32(self.vertices.len() as u32)?;

        if version >= 36 {
            stream.write_boolean(is_ng)?;

            if is_ng {
                // TODO: Determine if value changes after v37
                let vert_stride = 36;

                stream.write_uint32(vert_stride)?;
                stream.write_uint32(1)?; // Some constant
            }
        }

        // TODO: Remove once next gen vertex format is figured out
        if version >= 36 && is_ng && self.vertices.len() == self.raw_vertices.len() {
            // Write raw vertices
            for raw_vert in self.raw_vertices.iter() {
                stream.write_bytes(raw_vert)?;
            }
        } else {
            // Write vertices
            // TODO: Separate into functions and use conditionals before loop iteration
            for v in &self.vertices {
                // Position
                stream.write_float32(v.pos.x)?;
                stream.write_float32(v.pos.y)?;
                stream.write_float32(v.pos.z)?;
                if version == 34 {
                    stream.write_float32(v.pos.w)?;
                }

                if version < 35 || !is_ng {
                    // Normals
                    stream.write_float32(v.normals.x)?;
                    stream.write_float32(v.normals.y)?;
                    stream.write_float32(v.normals.z)?;
                    if version == 34 {
                        stream.write_float32(v.normals.w)?;
                    }

                    // Weights
                    stream.write_float32(v.weights[0])?;
                    stream.write_float32(v.weights[1])?;
                    stream.write_float32(v.weights[2])?;
                    stream.write_float32(v.weights[3])?;

                    // UVs
                    stream.write_float32(v.uv.u)?;
                    stream.write_float32(v.uv.v)?;

                    if version >= 34 {
                        // Bone indices
                        stream.write_uint16(v.bones[0])?;
                        stream.write_uint16(v.bones[1])?;
                        stream.write_uint16(v.bones[2])?;
                        stream.write_uint16(v.bones[3])?;

                        // Tangent?
                        stream.write_float32(v.tangent.x)?;
                        stream.write_float32(v.tangent.y)?;
                        stream.write_float32(v.tangent.z)?;
                        stream.write_float32(v.tangent.w)?;
                    }
                } else {
                    todo!("Figure out how ng verts are packed in v36 meshes");
                }
            }
        }

        stream.write_uint32(self.faces.len() as u32)?;
        for f in &self.faces {
            stream.write_uint16(f[0])?;
            stream.write_uint16(f[1])?;
            stream.write_uint16(f[2])?;
        }

        stream.write_uint32(self.face_groups.len() as u32)?;
        stream.write_bytes(self.face_groups.as_slice())?;

        if version >= 34 {
            // Write n-bones
            stream.write_uint32(self.bones.len() as u32)?;

            for b in &self.bones {
                stream.write_prefixed_string(&b.name)?;
                save_matrix(&b.trans, &mut stream)?;
            }
        } else {
            if self.bones.is_empty() {
                // Write 0 bones
                stream.write_uint32(0)?;
            } else {
                // Write 4 bones
                for i in 0..4 {
                    if let Some(b) = self.bones.get(i) {
                        // Write bone
                        stream.write_prefixed_string(&b.name)?;
                        save_matrix(&b.trans, &mut stream)?;
                    } else {
                        // Write empty bone
                        stream.write_uint32(0)?;
                        save_matrix(&Matrix::identity(), &mut stream)?;
                    }
                }

                todo!("Figure out how to calculate additional group info");
            }
        }

        if version >= 36 {
            stream.write_boolean(self.keep_mesh_data)?;
        }

        if version == 37 {
            stream.write_boolean(self.exclude_from_self_shadow)?;
        } else if version >= 38 {
            stream.write_boolean(self.has_ao_calculation)?;
        }

        Ok(())
    }
}

