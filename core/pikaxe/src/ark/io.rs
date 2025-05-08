use crate::ark::*;
use crate::io::*;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
#[cfg(feature = "python")] use pyo3::prelude::*;

const MAX_HDR_SIZE: u64 = 20 * 0x100000; // 20MB
const FREQ_ARK_VERSION: i32 = i32::from_le_bytes(*b"ARK\0");

impl Ark {
    pub fn from_path<T: AsRef<Path>>(path: T) -> Result<Ark, ArkReadError> {
        let path = path.as_ref();

        // Check if extension is .ark
        let is_ark = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .eq_ignore_ascii_case("ark");

        if is_ark {
            let (version_major, _version_minor) = peek_ark_version(path)?;

            if version_major == FREQ_ARK_VERSION {
                todo!("Arks from frequency is not supported")
            } else if ![1, 2].contains(&version_major) {
                return Err(ArkReadError::ArkVersionNotSupported {
                    version: version_major
                });
            }

            let mut ark = Ark {
                version: version_major,
                encryption: ArkEncryption::None,
                path: path.to_owned(),
                ..Default::default()
            };

            ark.parse_amp_ark()?;
            return Ok(ark);
        }

        let hdr_size = get_file_size(path);
        if hdr_size > MAX_HDR_SIZE {
            return Err(ArkReadError::HdrTooBig);
        }

        let mut hdr_data = read_to_bytes(path);
        let mut version = get_version(&hdr_data[0..4]);
        let mut crypt_key = None;

        if !version_is_supported(version) {
            // Decrypt hdr (use version as key)
            crypt_dtb_style(&mut hdr_data[4..], version, None);
            crypt_key = Some(version);

            version = get_version(&hdr_data[4..8]);

            // Check version compatibility again
            if !version_is_supported(version) {
                return Err(ArkReadError::ArkVersionNotSupported {
                    version
                });
            }
        }

        let mut ark = Ark {
            version,
            encryption: match crypt_key {
                Some(key) => ArkEncryption::ClassicEncryption(key),
                None => ArkEncryption::None,
            },
            path: path.to_owned(),
            part_paths: {
                let dir_path = path.parent().unwrap();
                let files = dir_path.find_files_with_depth(FileSearchDepth::Immediate).unwrap();

                let mut ark_parts = files
                    .into_iter()
                    .filter(|f| f
                        .as_os_str()
                        .to_str()
                        .is_some_and(|e| e.ends_with(".ARK") || e.ends_with(".ark")))
                    .collect::<Vec<_>>();

                ark_parts.sort_by(|a, b| a.cmp(b));
                ark_parts
            },
            ..Default::default()
        };

        let read_hdr = match ark.encryption {
            ArkEncryption::None => &hdr_data[..],
            _ => &hdr_data[4..],
        };

        ark.parse_header(read_hdr)?;
        Ok(ark)
    }

    fn parse_amp_ark(&mut self) -> Result<(), ArkReadError> {
        let mut stream = FileStream::from_path_as_read_open(&self.path)
            .map_err(|_| ArkReadError::ArkNotSupported)?;
        let mut reader = BinaryStream::from_stream(&mut stream);

        reader.seek(SeekFrom::Start(4)).unwrap();
        let entry_count = reader.read_uint32().unwrap();

        // Read strings
        reader.seek(SeekFrom::Current((20 * entry_count) as i64)).unwrap();

        let strings: Vec<String> = if self.version == 1 {
            // Read size-prefixed strings from OPM amp demo
            let string_count = reader.read_uint32()
                .map_err(|_| ArkReadError::ArkNotSupported)?;

            let mut strings = vec![String::new(); string_count as usize];

            for s in strings.iter_mut() {
                *s = reader.read_prefixed_string()
                    .map_err(|_| ArkReadError::ArkNotSupported)?;
            }

            strings
        } else {
            // Read strings from blob + string entries
            let mut strings = parse_string_blob(&mut reader)?;
            let string_indicies = parse_string_indices(&mut reader)?;

            string_indicies
                .into_iter()
                .map(|si| strings
                    .remove(&si)
                    .unwrap_or_default())
                .collect()
        };

        // Read entries
        reader.seek(SeekFrom::Start(8)).unwrap(); // Skip version + entry count
        for id in 0..entry_count {
            let mut offset = 0;

            if self.version != 1 {
                offset = reader.read_uint32()
                    .map_err(|_| ArkReadError::ArkNotSupported)?;
            }

            let file_name_idx = reader.read_uint32().map_err(|_| ArkReadError::ArkNotSupported)? as usize;
            let dir_path_idx = reader.read_uint32().map_err(|_| ArkReadError::ArkNotSupported)? as usize;

            if self.version == 1 {
                offset = reader.read_uint32()
                    .map_err(|_| ArkReadError::ArkNotSupported)?;
            }

            let size = reader.read_uint32().map_err(|_| ArkReadError::ArkNotSupported)? as usize;
            let inflated_size = reader.read_uint32().map_err(|_| ArkReadError::ArkNotSupported)? as usize;

            self.entries.push(ArkOffsetEntry {
                id,
                path: if dir_path_idx < strings.len() {
                    // Join dir path + file name
                    let file_name = &strings[file_name_idx];
                    let dir_path = &strings[dir_path_idx];

                    create_full_path(dir_path, file_name)
                } else {
                    // Take just file name
                    strings[file_name_idx].to_owned()
                },
                offset: offset as u64,
                part: 0,
                size,
                inflated_size,
            });
        }

        self.sort_entries_by_name();

        Ok(())
    }

    fn parse_header(&mut self, hdr: &[u8]) -> Result<(), ArkReadError> {
        let mut stream = MemoryStream::from_slice_as_read(hdr);
        let mut reader = BinaryStream::from_stream(&mut stream);

        // Skip read version
        reader.seek(SeekFrom::Current(4))
            .map_err(|_| ArkReadError::ArkNotSupported)?;

        let _part_count = reader.read_uint32()
            .map_err(|_| ArkReadError::ArkNotSupported)?;

        // TODO: Verify both counts match
        let part_size_count = reader.read_uint32()
            .map_err(|_| ArkReadError::ArkNotSupported)?;

        let mut part_size_ranges = vec![(0u64, 0u64); part_size_count as usize];
        let mut part_start = 0u64;

        // Read part sizes
        for p in part_size_ranges.iter_mut() {
            let size = reader.read_uint32()
                .map_err(|_| ArkReadError::ArkNotSupported)? as u64;

            *p = (part_start, part_start + size);
            part_start += size;
        }

        if self.version >= 5 {
            // TODO: Verify count matches here too
            let part_name_count = reader.read_uint32()
                .map_err(|_| ArkReadError::ArkNotSupported)?;

            // Skip part file names
            for _ in 0..part_name_count {
                reader.read_prefixed_string()
                    .map_err(|_| ArkReadError::ArkNotSupported)?;
            }
        }

        // Read string blob
        let strings = parse_string_blob(&mut reader)?;

        // Read string indicies
        let string_indicies = parse_string_indices(&mut reader)?;

        // Read file entries (and sort)
        self.parse_file_entries(&mut reader, part_size_ranges.as_slice(), &strings, string_indicies.as_slice())?;
        self.sort_entries_by_name();

        Ok(())
    }

    fn parse_file_entries(&mut self, reader: &mut BinaryStream, part_sizes: &[(u64, u64)], strings: &HashMap<u32, String>, string_indices: &[u32]) -> Result<(), ArkReadError> {
        let entry_count = reader.read_uint32()
            .map_err(|_| ArkReadError::ArkNotSupported)?;

        for id in 0..entry_count {
            // Read offset as either u32 or u64 depending on ark version
            let offset = match self.version {
                3 | 4 => reader.read_uint32().map_err(|_| ArkReadError::ArkNotSupported)? as u64,
                _ => reader.read_uint64().map_err(|_| ArkReadError::ArkNotSupported)?
            };

            let file_name_idx = reader.read_uint32().map_err(|_| ArkReadError::ArkNotSupported)? as usize;
            let dir_path_idx = reader.read_uint32().map_err(|_| ArkReadError::ArkNotSupported)? as usize;
            let size = reader.read_uint32().map_err(|_| ArkReadError::ArkNotSupported)? as usize;
            let inflated_size = reader.read_uint32().map_err(|_| ArkReadError::ArkNotSupported)? as usize;

            let file_name = &strings[&string_indices[file_name_idx]];
            let dir_path = &strings[&string_indices[dir_path_idx]];

            let (part, offset) = get_ark_part_and_offset(offset, part_sizes);

            self.entries.push(ArkOffsetEntry {
                id,
                path: create_full_path(dir_path, file_name),
                offset,
                part,
                size,
                inflated_size,
            });
        }

        Ok(())
    }

    fn sort_entries_by_name(&mut self) {
        self.entries.sort_by(|a, b| a.path.cmp(&b.path));
    }
}

fn get_version(data: &[u8]) -> i32 {
    let mut buffer = [0u8; 4];
    buffer.copy_from_slice(&data[0..4]);

    i32::from_le_bytes(buffer)
}

fn version_is_supported(version: i32) -> bool {
    match version {
        3 | 5 => true,
        _ => false
    }
}

fn parse_string_blob(reader: &mut BinaryStream) -> Result<HashMap<u32, String>, ArkReadError> {
    let mut strings = HashMap::new();
    let blob_size = reader.read_uint32()
        .map_err(|_| ArkReadError::ArkNotSupported)?;

    let mut offset = 0;
    let start_pos = reader.pos();

    // Read string from table
    while offset < blob_size {
        let s = reader.read_null_terminated_string()
            .map_err(|_| ArkReadError::ArkNotSupported)?;

        strings.insert(offset, s);
        offset = (reader.pos() - start_pos) as u32;
    }

    Ok(strings)
}

fn parse_string_indices(reader: &mut BinaryStream) -> Result<Vec<u32>, ArkReadError> {
    let indices_count = reader.read_uint32()
        .map_err(|_| ArkReadError::ArkNotSupported)?;

    let mut indices = vec![0; indices_count as usize];

    for ind in indices.iter_mut() {
        *ind = reader.read_uint32()
            .map_err(|_| ArkReadError::ArkNotSupported)?;
    }

    Ok(indices)
}

fn create_full_path(dir_path: &String, file_name: &String) -> String {
    if dir_path.is_empty() {
        return file_name.to_owned();
    }

    format!("{}/{}", dir_path, file_name)
}

fn get_ark_part_and_offset(offset: u64, part_size_ranges: &[(u64, u64)]) -> (u32, u64) {
    part_size_ranges
        .iter()
        .enumerate()
        .find(|(_, (start, end))| &offset >= start && &offset < end)
        .map(|(i, (start, _))| (i as u32, &offset - start))
        .unwrap()
}

fn peek_ark_version(path: &Path) -> Result<(i32, Option<i32>), ArkReadError> {
    let mut ark_file = std::fs::File::open(path)
        .map_err(|_| ArkReadError::ArkNotSupported)?;

    let v1 = {
        // Read ark version
        let mut buffer = [0u8; 4];

        ark_file.read_exact(&mut buffer)
            .map_err(|_| ArkReadError::ArkNotSupported)?;

        i32::from_le_bytes(buffer)
    };

    let v2 = if v1 == FREQ_ARK_VERSION {
        // Read minor version from freq ark
        let mut buffer = [0u8; 4];

        ark_file.read_exact(&mut buffer)
            .map_err(|_| ArkReadError::ArkNotSupported)?;

        Some(i32::from_le_bytes(buffer))
    } else {
        None
    };

    Ok((v1, v2))
}