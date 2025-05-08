use crate::io::{BinaryStream, SeekFrom, Stream};
use crate::dta::*;
use itertools::Itertools;
use thiserror::Error as ThisError;
use std::error::Error;
use std::hint::unreachable_unchecked;

#[derive(Debug, ThisError)]
pub enum DtaLoadError {
    #[error("Unknown node type: {node_type:#02X}")]
    UnknownNodeType {
        node_type: u32
    },
    #[error("Unknown dtb version: {version:#02X}")]
    UnknownVersion {
        version: u32
    },
}

#[derive(Debug)]
pub enum DataArrayIOSettings {
    Milo,
    Forge,
    Amplitude,
}

impl DataArray {
    pub fn load(&mut self, _stream: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

impl RootData {
    pub fn save(&self, stream: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
        let has_data = !self.data.is_empty();

        // Save data
        stream.write_boolean(has_data)?;
        if has_data {
            save_array(&self.data, stream, &mut 0)?;
        }

        Ok(())
    }

    pub fn load(&mut self, stream: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
        // Clear data
        self.data.clear();

        // Read data
        let has_data = stream.read_boolean()?;
        if has_data {
            self.data = load_array(stream)?;
        };

        Ok(())
    }

    pub fn load_with_settings(&mut self, stream: &mut Box<BinaryStream>, settings: DataArrayIOSettings) -> Result<(), Box<dyn Error>> {
        match settings {
            DataArrayIOSettings::Milo => {
                return self.load(stream);
            },
            DataArrayIOSettings::Forge => todo!("Can't read Forge dtb"),
            DataArrayIOSettings::Amplitude => {
                // Clear data
                self.data.clear();

                // Read data
                let data_version = stream.read_uint8()?;

                if data_version != 2 {
                    return Err(Box::new(DtaLoadError::UnknownVersion {
                        version: data_version as u32
                    }));
                }

                // Read original file names (ignore for now)
                let name_count = stream.read_uint32()?;
                for _ in 0..name_count {
                    stream.read_prefixed_string()?;
                }

                self.data = load_array_amp(stream)?;
                Ok(())
            }
        }
    }
}

pub(crate) fn save_array(data: &Vec<DataArray>, stream: &mut Box<BinaryStream>, id: &mut u32) -> Result<(), Box<dyn Error>> {
    stream.write_uint16(data.len() as u16)?;
    stream.write_uint32(*id)?;

    // Update id (actually line # in dta)
    *id += 1;

    for node in data {
        save_node(node, stream, id)?;
    }

    Ok(())
}

pub(crate) fn load_array_amp(stream: &mut Box<BinaryStream>) -> Result<Vec<DataArray>, Box<dyn Error>> {
    let count = stream.read_uint16()? as usize;
    let _id_1 = stream.read_uint32()?;
    let _id_2 = stream.read_uint32()?;

    // Types are packed in 2-bits, so 16 types per 32-bit word
    let mut type_count = count / 16;
    if (count % 16) > 0 {
        type_count += 1;
    }

    let mut types = vec![0u32; type_count];

    for typ in types.iter_mut() {
        *typ = stream.read_uint32()?;
    }

    let mut nodes = Vec::new();

    for i in 0..count {
        // Interpret type
        let div = i / 16;
        let rem = i % 16;
        let typ = (types[div] >> (rem * 2)) & 0x03;

        nodes.push(load_node_amp(stream, typ)?);
    }

    Ok(nodes)
}

pub(crate) fn load_array(stream: &mut Box<BinaryStream>) -> Result<Vec<DataArray>, Box<dyn Error>> {
    let count = stream.read_uint16()? as usize;
    let _id = stream.read_uint32()?;

    let mut nodes = Vec::new();

    for _ in 0..count {
        nodes.push(load_node(stream)?);
    }

    Ok(nodes)
}

fn save_node(data: &DataArray, writer: &mut Box<BinaryStream>, id: &mut u32) -> Result<(), Box<dyn Error>> {
    let node_enum = data.get_enum_value();
    writer.write_uint32(node_enum)?;

    match data {
        DataArray::Integer(int) => writer.write_int32(*int)?,
        DataArray::Float(f) => writer.write_float32(*f)?,
        DataArray::Variable(str) => save_string(str, writer)?,
        /*DataArray::Func(str) => {
            save_string(str, writer)?;
        },*/
        DataArray::Object(str) => save_string(str, writer)?,
        DataArray::Symbol(str) => save_string(str, writer)?,
        DataArray::KDataUnhandled => writer.write_int32(0)?,
        DataArray::IfDef(str) => save_string(str, writer)?,
        DataArray::Else => writer.write_int32(0)?,
        DataArray::EndIf => writer.write_int32(0)?,
        DataArray::Array(arr) => save_array(arr, writer, id)?,
        DataArray::Command(arr) => save_array(arr, writer, id)?,
        DataArray::String(str) => save_string(str, writer)?,
        DataArray::Property(arr) => save_array(arr, writer, id)?,
        DataArray::Define(str) => save_string(str, writer)?,
        DataArray::Include(str) => save_string(str, writer)?,
        DataArray::Merge(str) => save_string(str, writer)?,
        DataArray::IfNDef(str) => save_string(str, writer)?,
        DataArray::Autorun => writer.write_int32(0)?,
        DataArray::Undef(str) => save_string(str, writer)?
    };
    Ok(())
}

fn load_node(stream: &mut Box<BinaryStream>) -> Result<DataArray, Box<dyn Error>> {
    let node_type = stream.read_uint32()?;

    let node = match node_type {
        0x00 => DataArray::Integer(stream.read_int32()?),
        0x01 => DataArray::Float(stream.read_float32()?),
        0x02 => DataArray::Variable(load_string(stream)?),
        0x03 => todo!(), // TODO: Add func support
        0x04 => DataArray::Object(load_string(stream)?),
        0x05 => DataArray::Symbol(load_string(stream)?),
        0x06 => {
            stream.seek(SeekFrom::Current(4))?;
            DataArray::KDataUnhandled
        },
        0x07 => DataArray::IfDef(load_string(stream)?),
        0x08 => {
            stream.seek(SeekFrom::Current(4))?;
            DataArray::Else
        },
        0x09 => {
            stream.seek(SeekFrom::Current(4))?;
            DataArray::EndIf
        },
        0x10 => DataArray::Array(load_array(stream)?),
        0x11 => DataArray::Command(load_array(stream)?),
        0x12 => DataArray::String(load_string(stream)?),
        0x13 => DataArray::Property(load_array(stream)?),
        0x20 => DataArray::Define(load_string(stream)?),
        0x21 => DataArray::Include(load_string(stream)?),
        0x22 => DataArray::Merge(load_string(stream)?),
        0x23 => DataArray::IfNDef(load_string(stream)?),
        0x24 => {
            stream.seek(SeekFrom::Current(4))?;
            DataArray::Autorun
        },
        0x25 => DataArray::Undef(load_string(stream)?),
        _ => return Err(Box::new(DtaLoadError::UnknownNodeType {
            node_type
        }))
    };

    Ok(node)
}

fn load_node_amp(stream: &mut Box<BinaryStream>, node_type: u32) -> Result<DataArray, Box<dyn Error>> {
    let node = match node_type {
        0x00 => DataArray::Integer(stream.read_int32()?),
        0x01 => DataArray::Symbol(load_string(stream)?),
        0x02 => DataArray::Float(stream.read_float32()?),
        0x03 => DataArray::Array(load_array_amp(stream)?),
        _ => unreachable!("Shouldn't be reached. Node type of \"{node_type}\" is invalid"),
    };

    Ok(node)
}

fn save_string(str: &DataString, stream: &mut Box<BinaryStream>) -> Result<(), Box<dyn Error>> {
    let raw_data = str.get_raw();

    stream.write_uint32(raw_data.len() as u32)?;
    stream.write_bytes(raw_data.as_slice())?;

    Ok(())
}

fn load_string(stream: &mut Box<BinaryStream>) -> Result<DataString, Box<dyn Error>> {
    let length = stream.read_uint32()? as usize;
    Ok(stream.read_bytes(length)?.into())
}