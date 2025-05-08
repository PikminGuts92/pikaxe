use crate::SystemInfo;
use crate::io::compression::*;
use crate::io::stream::{BinaryStream, IOEndian, MemoryStream, SeekFrom, Stream};
use crate::scene::{Object, ObjectDir, ObjectDirBase, PackedObject};
use std::cmp::Ordering;
use std::error::Error;


use thiserror::Error as ThisError;

const MAX_BLOCK_SIZE: usize = 0x20000;
const ADDE_PADDING: [u8; 4] = [0xAD, 0xDE, 0xAD, 0xDE];
const GZIP_MAGIC: u32 = u32::from_le_bytes([0x1F, 0x8B, 0x08, 0x08]);

#[derive(Copy, Clone, Debug)]
pub enum BlockType
{
    TypeA, // Block structure, no compression
    TypeB, // Block structure, zlib compression
    TypeC, // Block structure, gzip compression
    TypeD, // Block structure, zlib compression (with inflate sizes block prefixed)
}

#[derive(Debug)]
pub struct BlockInfo {
    block_type: BlockType,
    start_offset: u32,
    block_sizes: Vec<usize>,
}

impl BlockInfo {
    pub fn new() -> BlockInfo {
        BlockInfo {
            block_type: BlockType::TypeB,
            start_offset: 2064,
            block_sizes: Vec::new()
        }
    }
}

#[derive(Debug)]
pub enum MiloArchiveStructure {
    Blocked(BlockInfo),
    GZIP,
    Uncompressed,
}

#[derive(Debug)]
pub struct MiloArchive {
    structure: MiloArchiveStructure,
    data: Vec<u8>
}

#[derive(Debug, ThisError)]
pub enum MiloBlockStructureError {
    #[error("Unsupported compression with magic of 0x{magic:X}")]
    UnsupportedCompression {
        magic: u32
    },
    #[error("UnknownIOError")] // TODO: Enclose IOError
    IOError
}

#[derive(Debug, ThisError)]
pub enum MiloUnpackError {
    #[error("Unsupported milo directory of version of {version}")]
    UnsupportedDirectoryVersion {
        version: u32
    }
}

impl MiloArchive {
    pub fn from_stream<T: Stream>(stream: &mut T) -> Result<MiloArchive, Box<dyn Error>> {
        let mut reader = BinaryStream::from_stream(stream); // Should always be little endian
        
        let mut structure: MiloArchiveStructure = MiloArchiveStructure::Uncompressed; // TODO: Handle in else case
        let mut uncompressed: Vec<u8> = Vec::new();

        let block_result = MiloArchive::get_block_type_or_none(&mut reader);

        if let Err(MiloBlockStructureError::UnsupportedCompression { magic }) = block_result {
            reader.seek(SeekFrom::Current(-4))?;

            if magic == GZIP_MAGIC {
                let mut data = vec![0u8; reader.len()?];
                reader.read_bytes_into_slice(&mut data)?;

                uncompressed = inflate_gzip_block_no_buffer(&data)?;
            } else {
                return Err(Box::new(block_result.unwrap_err()));
            }
        } else if let Ok(Some(block_type)) = block_result {
            let mut block_info = BlockInfo::new();

            block_info.block_type = block_type;
            block_info.start_offset = reader.read_uint32()?;

            let block_count = reader.read_int32()?;
            let max_inflate_size = reader.read_int32()?;

            let mut block_sizes: Vec<u32> = vec![0; block_count as usize];

            for size in block_sizes
                .iter_mut() {
                *size = reader.read_uint32()?;
            }

            // Advances to first block
            reader.seek(SeekFrom::Start(block_info.start_offset as u64))?;

            // Create buffer
            let mut buffer = vec![0u8; max_inflate_size as usize];

            for block_size in block_sizes.iter() {
                let bytes = reader.read_bytes((*block_size & 0xFFFFFF) as usize)?;

                let mut data = match (block_type, ((*block_size & 0xFF000000) == 0)) {
                    (BlockType::TypeA, _)
                        | (BlockType::TypeD, false) => bytes, // No compression
                    (BlockType::TypeB, _) => inflate_zlib_block(&bytes, &mut buffer[..])?,
                    (BlockType::TypeC, _) => inflate_gzip_block(&bytes, &mut buffer[..])?,
                    (BlockType::TypeD, true) => inflate_zlib_block(&bytes[4..], &mut buffer[..])?, // Skip 4-byte inflated size prefix
                };

                uncompressed.append(&mut data);
                block_info.block_sizes.push(data.len());
            }

            structure = MiloArchiveStructure::Blocked(block_info);
        } // TODO: Handle else case (should currently return error if not blocked)
        
        Ok(MiloArchive {
            structure,
            data: uncompressed
        })
    }

    fn get_block_type_or_none(reader: &mut BinaryStream) -> Result<Option<BlockType>, MiloBlockStructureError> {
        let magic = reader.read_uint32()
            .map_err(|_| MiloBlockStructureError::IOError)?;

        match magic {
            0xCABEDEAF => Ok(Some(BlockType::TypeA)),
            0xCBBEDEAF => Ok(Some(BlockType::TypeB)),
            0xCCBEDEAF => Ok(Some(BlockType::TypeC)),
            0xCDBEDEAF => Ok(Some(BlockType::TypeD)),
            // TODO: Assume uncompressed archive, or gzip then check version
            _ => Err(MiloBlockStructureError::UnsupportedCompression { magic })
        }
    }

    pub fn get_stream<'a>(&'a self) -> Box<dyn Stream + 'a> {
        let stream = MemoryStream::from_slice_as_read(&self.data);
        Box::new(stream)
    }

    pub fn unpack_directory(&self, info: &SystemInfo) -> Result<ObjectDir, Box<dyn Error>> {
        let mut stream = self.get_stream();
        let stream_size = stream.len().unwrap() as u64;

        let stream = stream.as_mut();
        let mut reader = BinaryStream::from_stream_with_endian(stream, info.endian);

        // Read and verify version
        let version = reader.read_uint32()?;
        if info.version != version {
            return Err(Box::new(MiloUnpackError::UnsupportedDirectoryVersion { version }));
        }

        let mut dir_type;
        let dir_name;

        if version >= 24 {
            // Read object dir name + type
            dir_type = reader.read_prefixed_string()?;
            dir_name = reader.read_prefixed_string()?;

            reader.seek(SeekFrom::Current(8))?; // Skip extra nums

            if version >= 32 {
                reader.seek(SeekFrom::Current(1))?; // Skip unknown bool
            }

            // Update class name
            ObjectDir::fix_class_name(version, &mut dir_type);
        } else {
            dir_type = String::new();
            dir_name = String::new();
        }

        let entry_count = reader.read_int32()?;
        let mut packed_entries: Vec<PackedObject> = Vec::new();

        // Parse entry types + names
        if version <= 6 {
            // Read as null-terminated strings
            for _ in 0..entry_count {
                let mut entry_type = reader.read_null_terminated_string()?;
                let entry_name = reader.read_null_terminated_string()?;

                reader.seek(SeekFrom::Current(1))?; // Unknown, always 1?

                // Update class name
                ObjectDir::fix_class_name(version, &mut entry_type);

                packed_entries.push(PackedObject {
                    name: entry_name,
                    object_type: entry_type,
                    data: Vec::new()
                })
            }
        } else {
            // Read as size-prefixed strings
            for _ in 0..entry_count {
                let mut entry_type = reader.read_prefixed_string()?;
                let entry_name = reader.read_prefixed_string()?;

                // Update class name
                ObjectDir::fix_class_name(version, &mut entry_type);

                packed_entries.push(PackedObject {
                    name: entry_name,
                    object_type: entry_type,
                    data: Vec::new()
                })
            }
        }

        if version == 10 {
            // Read external paths
            let ext_count = reader.read_int32()?;

            // TODO: Collect into struct
            for _ in 0..ext_count {
                reader.read_prefixed_string()?;
            }
        } else if version > 10 {
            // TODO: Parse directory info (entry)
            /*let entry_size = self.guess_entry_size(&mut reader)?.unwrap();
            reader.seek(SeekFrom::Current((entry_size + 4) as i64))?;*/

            // Hacky way to read directory entry
            // Only works if no sub dirs
            packed_entries.insert(0, PackedObject {
                name: dir_name.to_owned(),
                object_type: dir_type.to_owned(),
                data: Vec::new()
            });
        }

        // Get data for entries
        for entry_obj in packed_entries.iter_mut() {
            if let Some(size) = self.guess_entry_size(&mut reader)? {
                // Read data and skip padding
                entry_obj.data = reader.read_bytes(size)?;
                reader.seek(SeekFrom::Current(4))?;
            } else {
                // TODO: Else throw error?
                break;
            }
        }

        if stream.pos() < stream_size {
            log::warn!("Read less data than length of milo file. Likely not parsed correctly.");
        }

        Ok(ObjectDir::ObjectDir(ObjectDirBase {
            entries: packed_entries
                .into_iter()
                .map(Object::Packed)
                .collect(),
            name: dir_name,
            dir_type,
            sub_dirs: Vec::new()
        }))
    }

    fn guess_entry_size<'a>(&'a self, reader: &mut BinaryStream) -> Result<Option<usize>, Box<dyn Error>> {
        let start_pos = reader.pos();
        let stream_len = reader.len()?;

        let mut magic: i32;

        loop {
            if reader.seek_until(&ADDE_PADDING)?.is_none() {
                // End of file reached
                reader.seek(SeekFrom::Start(start_pos))?;
                return Ok(None);
            }

            // Found padding, skip needle bytes
            reader.seek(SeekFrom::Current(4))?;

            if (reader.pos() as usize) >= stream_len {
                // EOF reached
                break;
            }

            // Checks magic because ADDE padding can also be found in some Tex files as pixel data
            // This should reduce false positives
            magic = reader.read_int32()?;
            reader.seek(SeekFrom::Current(-4))?;

            if (0..=0xFF).contains(&magic) {
                break;
            }
        }

        // Calculates size and returns to start of stream
        let entry_size = (reader.pos() - (start_pos + 4)) as usize;
        reader.seek(SeekFrom::Start(start_pos))?;

        Ok(Some(entry_size))
    }

    fn get_type_order_value(obj_type: &str) -> i32 {
        // Same sort order in dta scripts
        match obj_type {
            "Tex" => 0,
            "Mat" => 1,
            "Font" => 2,
            "Text" => 3,
            "Mesh" => 4,
            "Blur" => 5,
            "Group" => 6,
            "View" => 6,
            "Trans" => 7,
            _ => 100
        }
    }

    fn compare_entries_by_type_and_name<'r, 's>(a: &'r &Object, b: &'s &Object) -> Ordering {
        // Get entry types
        let a_type = MiloArchive::get_type_order_value(a.get_type());
        let b_type = MiloArchive::get_type_order_value(b.get_type());

        // First compare type
        match a_type.cmp(&b_type) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => {
                // Get entry names
                let a_name = a.get_name();
                let b_name = b.get_name();

                // Then compare name
                a_name.cmp(b_name)
            }
        }
    }

    pub fn from_object_dir(obj_dir: &ObjectDir, info: &SystemInfo, block_type: Option<BlockType>) -> Result<MiloArchive, Box<dyn Error>> {
        // Create stream
        let mut data = Vec::<u8>::new();
        let mut stream = MemoryStream::from_vector_as_read_write(&mut data);
        let mut writer = BinaryStream::from_stream_with_endian(&mut stream, info.endian);

        let mut entries: Vec<&Object> = obj_dir.get_entries().iter().collect();

        // Write version
        writer.write_uint32(info.version)?;

        let mut dir_entry_op = None;
        if info.version >= 24 {
            // TODO: Refactor and get from field instead of hacky entries
            let dir_entry = entries.swap_remove(0); // Faster than remove(), will sort anyways

            // Write directory name + type
            let dir_type = dir_entry.get_type();
            let dir_name = dir_entry.get_name();

            writer.write_prefixed_string(dir_type)?;
            writer.write_prefixed_string(dir_name)?;

            // Compute values for string table
            let hash_count = (entries.len() + 1) * 2;
            let blob_size = entries
                .iter()
                .map(|o| o.get_name().len() + 1)
                .sum::<usize>() + (dir_name.len() + 1);

            // Write string table values
            writer.write_uint32(hash_count as u32)?;
            writer.write_uint32(blob_size as u32)?;

            dir_entry_op = Some(dir_entry);
        }

        writer.write_uint32(entries.len() as u32)?;

        // Write types + names
        entries.sort_by(MiloArchive::compare_entries_by_type_and_name);
        for entry in entries.iter() {
            let obj_type = entry.get_type();
            let obj_name = entry.get_name();

            writer.write_prefixed_string(obj_type)?;
            writer.write_prefixed_string(obj_name)?;
        }

        if info.version == 10 {
            // TODO: Determine external dependencies or get from directory property
            writer.write_uint32(0)?;
        } else {
            // Hacky way to write directory entry
            let dir_entry = dir_entry_op.unwrap();

            if let Object::Packed(packed) = dir_entry {
                writer.write_bytes(&packed.data.as_slice())?;
            }

            writer.write_bytes(&ADDE_PADDING)?;
        }

        let mut block_sizes = Vec::new();
        let mut current_size = writer.len()?;

        // Write data for entries
        for entry in entries.iter() {
            // Get packed entry
            match entry {
                Object::Packed(packed) => {
                    let data = &packed.data;

                    // Write to stream
                    writer.write_bytes(&data[..])?;
                    writer.write_bytes(&ADDE_PADDING)?;

                    // Update block size
                    current_size += data.len() + 4;
                },
                _ => {
                    // Pack entry
                    let data = entry
                        .pack(info)
                        .and_then(|o| match o {
                            Object::Packed(p) => Some(p.data),
                            _ => None
                        });

                    if let Some(data) = &data {
                        // Write to stream
                        writer.write_bytes(&data[..])?;
                        writer.write_bytes(&ADDE_PADDING)?;

                        // Update block size
                        current_size += data.len() + 4;
                    } else {
                        continue;
                    }
                }
            };

            if current_size >= MAX_BLOCK_SIZE {
                block_sizes.push(current_size);
                current_size = 0;
            }
        }

        if current_size > 0 {
            block_sizes.push(current_size);
        }

        Ok(MiloArchive {
            structure: MiloArchiveStructure::Blocked(BlockInfo {
                block_type: block_type.unwrap_or(BlockType::TypeB),
                start_offset: 2064,
                block_sizes
            }),
            data
        })
    }

    pub fn write_to_stream(&self, stream: &mut dyn Stream) -> Result<(), Box<dyn Error>> {
        let mut writer = BinaryStream::from_stream(stream);

        match &self.structure {
            MiloArchiveStructure::Blocked(info) => {
                // Get and write magic
                let magic: u32 = match &info.block_type {
                    BlockType::TypeA => 0xCABEDEAF,
                    BlockType::TypeB => 0xCBBEDEAF,
                    BlockType::TypeC => 0xCCBEDEAF,
                    BlockType::TypeD => 0xCDBEDEAF,
                };

                // Get max uncompressed size
                let max_block_size = match info.block_sizes.iter().max() {
                    Some(max) => *max,
                    None => 0
                };

                // Write infos
                writer.write_uint32(magic)?;
                writer.write_uint32(info.start_offset)?;
                writer.write_uint32(info.block_sizes.len() as u32)?;
                writer.write_uint32(max_block_size as u32)?;

                // Save current offset
                let block_sizes_offset = writer.pos();

                // TODO: Implement proper seek with insertion of empty bytes
                // Write empty bytes for now
                writer.write_bytes(&vec![0u8; (info.start_offset - 16) as usize][..])?;

                // Create buffer
                let mut buffer = vec![0u8; max_block_size];

                // Iterate over blocks and compress data
                let mut block_offset = 0;
                let mut deflate_sizes = Vec::new();
                for block_size in info.block_sizes.iter() {
                    let block_data = &self.data[block_offset..(block_offset + *block_size)];

                    if let BlockType::TypeA = &info.block_type {
                        // Write uncompressed block to stream
                        writer.write_bytes(block_data)?;

                        // Add uncompressed size
                        deflate_sizes.push(block_data.len());
                    }
                    else if let BlockType::TypeD = &info.block_type {
                        let compressed_data = &deflate_zlib_block(block_data, &mut buffer)?[..];

                        // Write compressed block to stream
                        writer.write_uint32(block_data.len() as u32)?; // Write inflated size
                        writer.write_bytes(compressed_data)?;

                        // Add compressed size
                        deflate_sizes.push(compressed_data.len() + 4);
                    } else {
                        let compressed_data = &deflate_zlib_block(block_data, &mut buffer)?[..];

                        // Write compressed block to stream
                        writer.write_bytes(compressed_data)?;

                        // Add compressed size
                        deflate_sizes.push(compressed_data.len());
                    }

                    // Update current offset
                    block_offset += *block_size;
                }

                // Go back to block sizes offset
                writer.seek(SeekFrom::Start(block_sizes_offset))?;

                // Write deflated sizes
                for size in deflate_sizes.iter() {
                    writer.write_uint32(*size as u32)?;
                }
            },
            MiloArchiveStructure::GZIP => {
                todo!("Gzip compression for milo archive not supported")
            }
            MiloArchiveStructure::Uncompressed => {
                // Write uncompressed data
                writer.write_bytes(&self.data[..])?;
            }
        }

        Ok(())
    }

    pub fn guess_endian_version(&self) -> Option<(IOEndian, u32)> {
        if self.data.len() < 4 {
            return None;
        }

        let mut buffer = [0u8; 4];
        buffer.copy_from_slice(&self.data[..4]);

        let mut endian = IOEndian::Big;
        let mut version = u32::from_be_bytes(buffer);
        if version > 32 {
            endian = IOEndian::Little;
            version = u32::from_le_bytes(buffer);
        }

        Some((endian, version))
    }

    pub fn get_version(&self, endian: IOEndian) -> Option<u32> {
        if self.data.len() < 4 {
            return None;
        }

        let mut buffer = [0u8; 4];
        buffer.copy_from_slice(&self.data[..4]);

        let version = match endian {
            IOEndian::Big => u32::from_be_bytes(buffer),
            _ => u32::from_le_bytes(buffer),
        };

        Some(version)
    }
}
