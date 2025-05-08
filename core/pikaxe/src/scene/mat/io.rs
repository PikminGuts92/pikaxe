use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use thiserror::Error as ThisError;
use std::error::Error;

#[derive(Debug, ThisError)]
pub enum MatLoadError {
    #[error("Mat version {version} is not supported")]
    MatVersionNotSupported {
        version: u32
    },
}

fn is_version_supported(version: u32) -> bool {
    match version {
        8 | 9 => true,        // Amp Demo/Amp
        15 => true,           // AntiGrav
        21 => true,           // GH1
        25 | 27 | 28 => true, // GH2 4-song/GH2/GH2 360
        41 | 47 => true,      // RB1/RB2
        55 | 56 => true,      // TBRB/GDRB
        68 => true,           // RB3
        _ => false
    }
}

impl ObjectReadWrite for MatObject {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;
        if !is_version_supported(version) {
            return Err(Box::new(MatLoadError::MatVersionNotSupported {
                version
            }));
        }

        load_object(self, &mut reader, info)?;

        // Amp/AntiGrav/GH1 mats can be linked to many textures
        if version <= 9 {
            // Read tex entries
            let tex_count = reader.read_uint32()?;

            let mut max_order = 0;

            for _ in 0..tex_count {
                let map_type = reader.read_uint32()?;

                // Skip unknown data
                reader.seek(SeekFrom::Current(61))?;

                // Set name
                let name = reader.read_prefixed_string()?;
                if map_type >= max_order {
                    max_order = map_type;
                    self.diffuse_tex = name;
                }

                /*match map_type {
                    2 => self.emissive_map = name,
                    3 | 4 => self.diffuse_tex = name,
                    5 => self.environ_map = name,
                    _ => continue,
                };*/
            }

            // Skip remaining unknown crap
            return Ok(());
        } else if version <= 21 {
            // Read tex entries
            let tex_count = reader.read_uint32()?;

            for _ in 0..tex_count {
                reader.read_uint32()?; // Skip unknown

                let map_type = reader.read_uint32()?;

                if map_type == 0 {
                    load_matrix(&mut self.tex_xfm, &mut reader)?;
                    self.tex_wrap = reader.read_uint32()?.into();
                } else {
                    // Skip transform + tex_wrap
                    reader.seek(SeekFrom::Current(48))?;
                    reader.seek(SeekFrom::Current(4))?;
                }

                // Set name
                let name = reader.read_prefixed_string()?;
                match map_type {
                    0 => self.diffuse_tex = name,
                    2 => self.environ_map = name,
                    _ => continue,
                };
            }

            if version <= 15 {
                // AntiGrav - Skip remaining unknown crap
                return Ok(());
            }
        }

        self.blend = reader.read_uint32()?.into();
        load_color3(&mut self.color, &mut reader)?;
        self.alpha = reader.read_float32()?;

        if version <= 21 {
            // TODO: Parse other info
            return Ok(());
        }

        self.prelit = reader.read_boolean()?;
        self.use_environ = reader.read_boolean()?;

        self.z_mode = reader.read_uint32()?.into();
        self.alpha_cut = reader.read_boolean()?;

        if version > 37 {
            self.alpha_threshold = reader.read_uint32()?;
        }
        self.alpha_write = reader.read_boolean()?;

        self.tex_gen = reader.read_uint32()?.into();
        self.tex_wrap = reader.read_uint32()?.into();

        load_matrix(&mut self.tex_xfm, &mut reader)?;
        self.diffuse_tex = reader.read_prefixed_string()?;

        self.next_pass = reader.read_prefixed_string()?;
        self.intensify = reader.read_boolean()?;
        self.cull = reader.read_boolean()?;

        self.emissive_multiplier = reader.read_float32()?;
        load_color3(&mut self.specular_rgb, &mut reader)?;
        self.specular_power = reader.read_float32()?;

        self.normal_map = reader.read_prefixed_string()?;
        self.emissive_map = reader.read_prefixed_string()?;
        self.specular_map = reader.read_prefixed_string()?;

        if version < 51 {
            // Seems to match diffuse_tex
            let _some_string = reader.read_prefixed_string()?;
            /*if !some_string.is_empty() {
                panic!("Some string is note empty with value: {}", &some_string);
            }*/
        }

        self.environ_map = reader.read_prefixed_string()?;

        // TODO: Figure out what this does
        /*if version < 37 {

        }*/

        if version > 25 {
            if version <= 55 || version > 56 {
                // Read as boolean
                self.per_pixel_lit = match reader.read_boolean()? {
                    false => PerPixel::kPerPixelOff,
                    _ => PerPixel::kPerPixelAllNgPlatforms,
                }
            } else {
                // Read as enum for GDRB
                self.per_pixel_lit = reader.read_uint32()?.into();
            }
        }

        // TODO: Reverse RB3 mat
        if version >= 68 {
            return Ok(());
        }

        if version >= 27 && version < 50 {
            // Ignore bool
            reader.read_boolean()?;
        }

        if version > 27 {
            self.stencil_mode = reader.read_uint32()?.into();
        }

        if version >= 29 && version < 41 {
            // Ignore string
            reader.read_prefixed_string()?;
        }

        if version >= 33 {
            self.fur = reader.read_prefixed_string()?;
        } else if version > 29 {
            // TODO: Load fur from "{mat_base_name}.fur" file
        }

        if version >= 34 && version < 49 {
            // Ignore bool, color, alpha
            reader.seek(SeekFrom::Current(17))?;

            if version > 34 {
                let some_string = reader.read_prefixed_string()?;
                if !some_string.is_empty() {
                    panic!("Some string is note empty with value: {}", &some_string);
                }
            }
        }

        if version > 35 {
            self.de_normal = reader.read_float32()?;
            self.anisotropy = reader.read_float32()?;
        }

        if version > 38 {
            if version < 42 {
                // Ignore bool
                reader.read_boolean()?;
            }

            self.norm_detail_tiling = reader.read_float32()?;
            self.norm_detail_strength = reader.read_float32()?;

            if version < 42 {
                // Ignore 5 floats
                reader.seek(SeekFrom::Current(20))?;
            }

            self.norm_detail_map = reader.read_prefixed_string()?;

            if version < 42 {
                let some_string = reader.read_prefixed_string()?;
                if !some_string.is_empty() {
                    panic!("Some string is note empty with value: {}", &some_string);
                }
            }

            // TODO: Figure out if this logic is right
            /*if version < 40 {
                self.norm_detail_strength = 0.0;
            }*/
        }

        if version > 42 {
            if version < 45 {
                let bitfield = reader.read_uint32()?;
                todo!("Parse point_lights from {}", bitfield);
            } else {
                self.point_lights = reader.read_boolean()?;
            }

            self.proj_lights = reader.read_boolean()?;
            self.fog = reader.read_boolean()?;
            self.fade_out = reader.read_boolean()?;

            if version > 46 {
                self.color_adjust = reader.read_boolean()?;
            }
        }

        if version > 47 {
            load_color3(&mut self.rim_rgb, &mut reader)?;
            self.rim_power = reader.read_float32()?;
            self.rim_map = reader.read_prefixed_string()?;
            self.rim_always_show = reader.read_boolean()?;
        }

        if version > 48 {
            self.screen_aligned = reader.read_boolean()?;
        }

        if version == 50 {
            let legacy_shader_variation = reader.read_uint8()?;

            self.shader_variation = match legacy_shader_variation {
                0 => ShaderVariation::kShaderVariationNone,
                _ => ShaderVariation::kShaderVariationSkin,
            };
        } else if version > 50 {
            self.shader_variation = reader.read_uint32()?.into();
            load_color3(&mut self.specular2_rgb, &mut reader)?;
            self.specular2_power = reader.read_float32()?;
        }

        if version > 51 {
            if version < 53 {
                // Ignore bool
                reader.read_boolean()?;
            } else {
                self.unknown_1 = reader.read_float32()?;
            }

            if version > 54 {
                self.unknown_2 = reader.read_float32()?;
                self.unknown_3 = reader.read_float32()?;
                self.unknown_4 = reader.read_float32()?;
                self.unknown_5 = reader.read_float32()?;
            }
        }

        if version > 53 {
            self.alpha_mask = reader.read_prefixed_string()?;
        }

        if version > 54 {
            self.ps3_force_trilinear = reader.read_boolean()?;
        }

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut stream = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 28;

        stream.write_uint32(version)?;

        save_object(self, &mut stream, info)?;

        if version <= 21 {
            todo!("Saving not implemented for v21 materials");
        }

        stream.write_uint32(self.blend as u32)?;
        save_color3(&self.color, &mut stream)?;
        stream.write_float32(self.alpha)?;

        if version <= 21 {
            // TODO: Write other info
            return Ok(());
        }

        stream.write_boolean(self.prelit)?;
        stream.write_boolean(self.use_environ)?;

        stream.write_uint32(self.z_mode as u32)?;
        stream.write_boolean(self.alpha_cut)?;

        if version > 37 {
            stream.write_uint32(self.alpha_threshold)?;
        }
        stream.write_boolean(self.alpha_write)?;

        stream.write_uint32(self.tex_gen as u32)?;
        stream.write_uint32(self.tex_wrap as u32)?;

        save_matrix(&self.tex_xfm, &mut stream)?;
        stream.write_prefixed_string(&self.diffuse_tex)?;

        stream.write_prefixed_string(&self.next_pass)?;
        stream.write_boolean(self.intensify)?;
        stream.write_boolean(self.cull)?;

        stream.write_float32(self.emissive_multiplier)?;
        save_color3(&self.specular_rgb, &mut stream)?;
        stream.write_float32(self.specular_power)?;

        stream.write_prefixed_string(&self.normal_map)?;
        stream.write_prefixed_string(&self.emissive_map)?;
        stream.write_prefixed_string(&self.specular_map)?;

        if version < 51 {
            // Not sure, some string
            stream.write_uint32(0)?;
        }

        stream.write_prefixed_string(&self.environ_map)?;

        if version > 25 {
            if version <= 55 {
                // Write as boolean
                let per_pixel = match &self.per_pixel_lit {
                    &PerPixel::kPerPixelOff => false,
                    _ => true
                };

                stream.write_boolean(per_pixel)?;
            } else {
                // Write as enum
                stream.write_uint32(self.per_pixel_lit as u32)?;
            }
        }

        if version >= 27 && version < 50 {
            // Ignore bool
            stream.write_boolean(false)?;
        }

        if version > 27 {
            stream.write_uint32(self.stencil_mode as u32)?;
        } else {
            return Ok(()); // Exit early
        }

        if version >= 29 && version < 41 {
            // Ignore string
            stream.write_uint32(0)?;
        }

        if version >= 33 {
            stream.write_prefixed_string(&self.fur)?;
        } else if version > 29 {
            // TODO: Load fur from "{mat_base_name}.fur" file?
        }

        if version >= 34 && version < 49 {
            // Ignore bool, color, alpha
            stream.write_boolean(false)?;
            save_color3(&Color3::white(), &mut stream)?;
            stream.write_float32(1.0)?;

            if version > 34 {
                // Some string
                stream.write_uint32(0)?;
            }
        }

        if version > 35 {
            stream.write_float32(self.de_normal)?;
            stream.write_float32(self.anisotropy)?;
        }

        if version > 38 {
            if version < 42 {
                // Ignore bool
                stream.write_boolean(false)?;
            }

            stream.write_float32(self.norm_detail_tiling)?;
            stream.write_float32(self.norm_detail_strength)?;

            if version < 42 {
                // Ignore 5 floats
                stream.write_float32(0.25)?;
                save_color3(&Color3::white(), &mut stream)?;
                stream.write_float32(1.0)?;
            }

            stream.write_prefixed_string(&self.norm_detail_map)?;

            if version < 42 {
                // Some string
                stream.write_uint32(0)?;
            }
        }

        if version > 42 {
            if version < 45 {
                // Write as bitfield
                stream.write_uint32(self.point_lights as u32)?;
            } else {
                // Write as boolean
                stream.write_boolean(self.point_lights)?;
            }

            stream.write_boolean(self.proj_lights)?;
            stream.write_boolean(self.fog)?;
            stream.write_boolean(self.fade_out)?;

            if version > 46 {
                stream.write_boolean(self.color_adjust)?;
            }
        }

        if version > 47 {
            save_color3(&self.rim_rgb, &mut stream)?;
            stream.write_float32(self.rim_power)?;
            stream.write_prefixed_string(&self.rim_map)?;
            stream.write_boolean(self.rim_always_show)?;
        }

        if version > 48 {
            stream.write_boolean(self.screen_aligned)?;
        }

        if version == 50 {
            // Write as boolean
            let legacy_shader_variation = match &self.shader_variation {
                ShaderVariation::kShaderVariationNone => 0u8,
                _ => 1u8,
            };

            stream.write_uint8(legacy_shader_variation)?;
        } else if version > 50 {
            // Write as enum
            stream.write_uint32(self.shader_variation as u32)?;

            save_color3(&self.specular2_rgb, &mut stream)?;
            stream.write_float32(self.specular2_power)?;
        }

        if version > 51 {
            if version < 53 {
                // Ignore bool
                stream.write_boolean(false)?;
            } else {
                stream.write_float32(self.unknown_1)?;
            }

            if version > 54 {
                stream.write_float32(self.unknown_2)?;
                stream.write_float32(self.unknown_3)?;
                stream.write_float32(self.unknown_4)?;
                stream.write_float32(self.unknown_5)?;
            }
        }

        if version > 53 {
            stream.write_prefixed_string(&self.alpha_mask)?;
        }

        if version > 54 {
            stream.write_boolean(self.ps3_force_trilinear)?;
        }

        Ok(())
    }
}