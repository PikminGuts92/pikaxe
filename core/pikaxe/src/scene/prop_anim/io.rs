use crate::dta::{DataArray, RootData, save_array};
use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::scene::*;
use crate::SystemInfo;
use pikaxe_traits::scene::*;
use std::error::Error;

fn is_version_supported(version: u32) -> bool {
    match version {
        11 | 12 => true,
        _ => false
    }
}

impl ObjectReadWrite for PropAnim {
    fn load(&mut self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut reader = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        let version = reader.read_uint32()?;
        if !is_version_supported(version) {
            // TODO: Switch to custom error
            panic!("PropAnim version \"{}\" is not supported!", version);
        }

        load_object(self, &mut reader, info)?;
        load_anim(self, &mut reader, info, false)?;

        if version >= 12 {
            self.unknown_toggle = reader.read_boolean()?;
        }

        // Reset prop keys
        self.keys.clear();

        let prop_keys_count = reader.read_uint32()?;
        for _ in 0..prop_keys_count {
            let prop_keys = load_prop_keys(&mut reader)?;
            self.keys.push(prop_keys);
        }

        Ok(())
    }

    fn save(&self, stream: &mut dyn Stream, info: &SystemInfo) -> Result<(), Box<dyn Error>> {
        let mut writer = Box::new(BinaryStream::from_stream_with_endian(stream, info.endian));

        // TODO: Get version from system info
        let version = 11;

        writer.write_uint32(version)?;

        save_object(self, &mut writer, info)?;
        save_anim(self, &mut writer, info, false)?;

        if version >= 12 {
            writer.write_boolean(self.unknown_toggle)?;
        }

        writer.write_uint32(self.keys.len() as u32)?;
        for prop_keys in self.keys.iter() {
            save_prop_keys(prop_keys, &mut writer)?;
        } 

        Ok(())
    }
}

fn load_prop_keys(reader: &mut Box<BinaryStream>) -> Result<PropKeys, Box<dyn Error>> {
    let type_1 = reader.read_uint32()?;
    let type_2 = reader.read_uint32()?;

    if type_1 != type_2 {
        panic!("Type 1 of {type_1} doesn't match type 2 {type_2} for key in PropAnim")
    }

    let mut keys: PropKeys = Default::default();

    keys.target = reader.read_prefixed_string()?;

    // Read dta
    keys.property = {
        let mut root_dta = RootData::new();
        root_dta.load(reader)?;

        root_dta.data
    };

    keys.interpolation = reader.read_uint32()?;
    keys.interp_handler = reader.read_prefixed_string()?;
    keys.unknown_enum = reader.read_uint32()?;

    // Read events
    let event_count = reader.read_uint32()?;
    keys.events = PropKeysEvents::from_enum_value(type_1);

    match &mut keys.events {
        PropKeysEvents::Float(events)   => {
            for _ in 0..event_count {
                let mut ev = AnimEventFloat::default();
                ev.value = reader.read_float32()?;
                ev.pos = reader.read_float32()?;

                events.push(ev);
            }
        },
        PropKeysEvents::Color(events) => {
            for _ in 0..event_count {
                let mut ev = AnimEventColor::default();
                ev.value = {
                    let mut color = Color4::default();
                    load_color4(&mut color, reader)?;

                    color
                };
                ev.pos = reader.read_float32()?;

                events.push(ev);
            }
        },
        PropKeysEvents::Object(events) => {
            for _ in 0..event_count {
                let mut ev = AnimEventObject::default();
                ev.text1 = reader.read_prefixed_string()?;
                ev.text2 = reader.read_prefixed_string()?;
                ev.pos = reader.read_float32()?;

                events.push(ev);
            }
        },
        PropKeysEvents::Bool(events) => {
            for _ in 0..event_count {
                let mut ev = AnimEventBool::default();
                ev.value = reader.read_boolean()?;
                ev.pos = reader.read_float32()?;

                events.push(ev);
            }
        },
        PropKeysEvents::Quat(events) => {
            for _ in 0..event_count {
                let mut ev = AnimEventQuat::default();
                ev.value = {
                    let mut quat = Quat::default();
                    load_quat(&mut quat, reader)?;

                    quat
                };
                ev.pos = reader.read_float32()?;

                events.push(ev);
            }
        },
        PropKeysEvents::Vector3(events) => {
            for _ in 0..event_count {
                let mut ev = AnimEventVector3::default();
                ev.value = {
                    let mut vector = Vector3::default();
                    load_vector3(&mut vector, reader)?;

                    vector
                };
                ev.pos = reader.read_float32()?;

                events.push(ev);
            }
        },
        PropKeysEvents::Symbol(events) => {
            for _ in 0..event_count {
                let mut ev = AnimEventSymbol::default();
                ev.text = reader.read_prefixed_string()?;
                ev.pos = reader.read_float32()?;

                events.push(ev);
            }
        }
    }

    Ok(keys)
}

fn save_prop_keys(keys: &PropKeys, writer: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    let type_value = keys.events.get_enum_value();

    // Write twice because milo is weird
    writer.write_uint32(type_value)?;
    writer.write_uint32(type_value)?;

    writer.write_prefixed_string(&keys.target)?;

    // Write dta
    // TODO: Move logic to dta io
    if keys.property.is_empty() {
        writer.write_boolean(false)?;
    } else {
        let mut id = 0;

        writer.write_boolean(true)?;
        save_array(&keys.property, writer, &mut id)?;
    }

    writer.write_uint32(keys.interpolation)?;
    writer.write_prefixed_string(&keys.interp_handler)?;
    writer.write_uint32(keys.unknown_enum)?;

    // Write events
    writer.write_uint32(keys.events.len() as u32)?;

    match &keys.events {
        PropKeysEvents::Float(events)   => {
            for ev in events {
                writer.write_float32(ev.value)?;
                writer.write_float32(ev.pos)?;
            }
        },
        PropKeysEvents::Color(events) => {
            for ev in events {
                save_color4(&ev.value, writer)?;
                writer.write_float32(ev.pos)?;
            }
        },
        PropKeysEvents::Object(events) => {
            for ev in events {
                writer.write_prefixed_string(&ev.text1)?;
                writer.write_prefixed_string(&ev.text2)?;
                writer.write_float32(ev.pos)?;
            }
        },
        PropKeysEvents::Bool(events) => {
            for ev in events {
                writer.write_boolean(ev.value)?;
                writer.write_float32(ev.pos)?;
            }
        },
        PropKeysEvents::Quat(events) => {
            for ev in events {
                save_quat(&ev.value, writer)?;
                writer.write_float32(ev.pos)?;
            }
        },
        PropKeysEvents::Vector3(events) => {
            for ev in events {
                save_vector3(&ev.value, writer)?;
                writer.write_float32(ev.pos)?;
            }
        },
        PropKeysEvents::Symbol(events) => {
            for ev in events {
                writer.write_prefixed_string(&ev.text)?;
                writer.write_float32(ev.pos)?;
            }
        }
    }

    Ok(())
}