use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
pub use std::io::SeekFrom;
use std::path::Path;
pub use half::f16;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IOEndian {
    Little,
    Big,
}

pub trait Stream {
    // io
    fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>, Box<dyn Error>>;
    fn read_bytes_into_slice(&mut self, buffer: &mut [u8]) -> Result<(), Box<dyn Error>>;
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>>;

    // Getters
    fn pos(&self) -> u64;
    fn can_write(&self) -> bool;
    fn len(&mut self) -> Result<usize, Box<dyn Error>>;
    
    fn seek(&mut self, pos: SeekFrom) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug)]
enum MemoryData<'a> {
    Read(&'a [u8]),
    ReadWrite(&'a mut Vec<u8>),
    ReadWriteOwned(Vec<u8>),
}

#[derive(Debug)]
pub struct MemoryStream<'a> {
    endian: IOEndian,
    position: u64,
    data: MemoryData<'a>
}

impl<'a> MemoryStream<'a> {
    pub fn new() -> MemoryStream<'a> {
        MemoryStream {
            endian: IOEndian::Little,
            position: 0,
            data: MemoryData::ReadWriteOwned(Vec::new())
        }
    }

    pub fn from_vector_as_read_write(data: &'a mut Vec<u8>) -> MemoryStream<'a> {
        MemoryStream {
            endian: IOEndian::Little,
            position: 0,
            data: MemoryData::ReadWrite(data)
        }
    }

    pub fn from_slice_as_read(data: &[u8]) -> MemoryStream {
        MemoryStream {
            endian: IOEndian::Little,
            position: 0,
            data: MemoryData::Read(data)
        }
    }

    // TODO:: Safely handle possible panic
    fn get_slice(&'a self, pos: u64, size: usize) -> &'a [u8] {
        let pos = pos as usize;

        match &self.data {
            MemoryData::Read(data) => {
                &(*data)[pos..(pos + size)]
            },
            MemoryData::ReadWrite(vec) => {
                &(*vec)[pos..(pos + size)]
            },
            MemoryData::ReadWriteOwned(vec) => {
                &vec[pos..(pos + size)]
            }
        }
    }
}

impl<'a> Stream for MemoryStream<'a> {
    fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buffer_vec = vec![0u8; length];
        let data = self.get_slice(self.position, length);

        buffer_vec.clone_from_slice(data); // TODO:: Safely handle possible panic
        self.position += length as u64;

        Ok(buffer_vec)
    }

    fn read_bytes_into_slice(&mut self, buffer: &mut [u8]) -> Result<(), Box<dyn Error>> {
        let data = self.get_slice(self.position, buffer.len());

        buffer.clone_from_slice(data); // TODO:: Safely handle possible panic
        self.position += buffer.len() as u64;

        Ok(())
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        // TODO: Switch to match expression
        let &mut vec_data;

        if let MemoryData::ReadWriteOwned(vec) = &mut self.data {
            vec_data = vec;
        } else if let MemoryData::ReadWrite(vec) = &mut self.data {
            vec_data = vec;
        } else {
            panic!("Not implmented yet") // TODO: Throw error (but it shouldn't reach this part)
        }

        let data_len = data.len();

        if self.position == vec_data.len() as u64 {
            vec_data.extend_from_slice(data);
        } else {
            let pos = self.position as usize;

            let bytes_left = vec_data.len() - pos;
            if bytes_left < data_len {
                // Add missing bytes
                let rem_bytes = vec![0u8; data_len - bytes_left];
                vec_data.extend_from_slice(&rem_bytes);
            }

            vec_data[pos..(pos + data_len)].clone_from_slice(data); // TODO:: Safely handle possible panic
        }

        self.position += data_len as u64;
        Ok(())
    }

    fn pos(&self) -> u64 {
        self.position
    }

    fn can_write(&self) -> bool {
        match &self.data {
            MemoryData::Read(_) => false,
            MemoryData::ReadWrite(_) => true,
            MemoryData::ReadWriteOwned(_) => true
        }
    }

    fn len(&mut self) -> Result<usize, Box<dyn Error>> {
        match &self.data {
            MemoryData::Read(data) => {
                Ok((*data).len())
            },
            MemoryData::ReadWrite(vec) => {
                Ok((*vec).len())
            },
            MemoryData::ReadWriteOwned(vec) => {
                Ok(vec.len())
            }
        }
    }

    fn seek(&mut self, pos: SeekFrom) -> Result<(), Box<dyn Error>> {
        self.position = match pos {
            SeekFrom::Start(rel_str) => rel_str,
            SeekFrom::End(rel_end) => ((self.len()? as i64) + rel_end) as u64,
            SeekFrom::Current(rel_cur) => ((self.position as i64) + rel_cur) as u64,
        };
        Ok(())
    }
}

#[derive(Debug)]
pub struct FileStream {
    position: u64,
    file: File,
    writeable: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct FileOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
}

impl FileStream {
    fn from_options(path: &Path, ops: FileOptions) -> Result<FileStream, Box<dyn Error>> {
        let file = OpenOptions::new()
            .read(ops.read)
            .write(ops.write)
            .create(ops.create)
            .truncate(ops.create && ops.write)
            .open(path)?;
        
        Ok(FileStream {
            position: 0,
            file,
            writeable: ops.write
        })
    }

    pub fn new(path: &Path, options: FileOptions) -> Result<FileStream, Box<dyn Error>> {
        Self::from_options(path, options)
    }

    pub fn from_path_as_read_open(path: &Path) -> Result<FileStream, Box<dyn Error>> {
        let file = File::open(path)?;

        Ok(FileStream {
            position: 0,
            file,
            writeable: false
        })
    }

    pub fn from_path_as_read_write_create(path: &Path) -> Result<FileStream, Box<dyn Error>> {
        Self::from_options(path, FileOptions {
            read: true,
            write: true,
            create: true
        })
    }

    pub fn from_path_as_write_create(path: &Path) -> Result<FileStream, Box<dyn Error>> {
        Self::from_options(path, FileOptions {
            read: false,
            write: true,
            create: true
        })
    }
}

impl Stream for FileStream {
    fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buffer_vec = vec![0u8; length];
        self.file.read_exact(&mut buffer_vec)?;
        self.position += length as u64;

        Ok(buffer_vec)
    }

    fn read_bytes_into_slice(&mut self, buffer: &mut [u8]) -> Result<(), Box<dyn Error>> {
        self.file.read_exact(buffer)?;
        self.position += buffer.len() as u64;

        Ok(())
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        if !self.can_write() {
            panic!("File stream is read-only"); // TODO: Throw error instead
        }

        self.file.write_all(data)?;
        self.position += data.len() as u64;
        Ok(())
    }

    fn pos(&self) -> u64 {
        self.position
    }

    fn can_write(&self) -> bool {
        self.writeable
    }

    fn len(&mut self) -> Result<usize, Box<dyn Error>> {
        let start_pos = self.pos();
        let size_result = self.file.seek(SeekFrom::End(0));

        self.seek(SeekFrom::Start(start_pos))?;

        match size_result {
            Ok(size) => {
                Ok(size as usize)
            },
            Err(err) => {
                Err(Box::new(err))
            }
        }
    }

    fn seek(&mut self, pos: SeekFrom) -> Result<(), Box<dyn Error>> {
        self.position = self.file.seek(pos)?;
        Ok(())
    }
}

pub struct BinaryStream<'a> {
    endian: IOEndian,
    stream: &'a mut dyn Stream
}

impl<'a> Stream for BinaryStream<'a> {
    fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        self.stream.read_bytes(length)
    }

    fn read_bytes_into_slice(&mut self, buffer: &mut [u8]) -> Result<(), Box<dyn Error>> {
        self.stream.read_bytes_into_slice(buffer)
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.stream.write_bytes(data)
    }

    fn pos(&self) -> u64 {
        self.stream.pos()
    }

    fn can_write(&self) -> bool {
        self.stream.can_write()
    }

    fn len(&mut self) -> Result<usize, Box<dyn Error>> {
        self.stream.len()
    }

    fn seek(&mut self, pos: SeekFrom) -> Result<(), Box<dyn Error>> {
        self.stream.seek(pos)
    }
}

impl<'a> BinaryStream<'a> {
    pub fn from_stream(stream: &mut dyn Stream) -> BinaryStream {
        BinaryStream {
            endian: IOEndian::Little,
            stream
        }
    }

    pub fn from_stream_with_endian(stream: &mut dyn Stream, endian: IOEndian) -> BinaryStream {
        BinaryStream {
            endian,
            stream
        }
    }

    // Getters
    pub fn endian(&self) -> IOEndian {
        self.endian
    }

    pub fn seek_until(&mut self, needle: &[u8]) -> Result<Option<usize>, Box<dyn Error>> {
        let start_pos = self.pos();
        let stream_len = self.len()?;

        let needle_len = needle.len();
        let search_limit = stream_len - needle_len;

        let mut haystack = vec![0u8; needle_len];
        while self.pos() <= search_limit as u64 {
            self.read_bytes_into_slice(&mut haystack[..])?;

            if haystack == needle {
                // Data found
                self.seek(SeekFrom::Current(-(needle_len as i64)))?;

                return Ok(Some((self.pos() - start_pos) as usize));
            } else {
                // Still searching
                self.seek(SeekFrom::Current(-((needle_len - 1) as i64)))?;
            }
        }

        Ok(None)
    }

    // Setters
    pub fn set_endian(&mut self, endian: IOEndian) {
        self.endian = endian;
    }
}

// Reader implementation
impl<'a> BinaryStream<'a> {
    // Read boolean
    pub fn read_boolean(&mut self) -> Result<bool, Box<dyn Error>> {
        let value = self.read_uint8()?;
        Ok(value != 0)
    }

    // Read signed integers
    pub fn read_int8(&mut self) -> Result<i8, Box<dyn Error>> {
        let mut buffer = [0u8; 1];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(i8::from_le_bytes(buffer)),
            IOEndian::Big => Ok(i8::from_be_bytes(buffer)),
        }
    }

    pub fn read_int16(&mut self) -> Result<i16, Box<dyn Error>> {
        let mut buffer = [0u8; 2];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(i16::from_le_bytes(buffer)),
            IOEndian::Big => Ok(i16::from_be_bytes(buffer)),
        }
    }

    pub fn read_int32(&mut self) -> Result<i32, Box<dyn Error>> {
        let mut buffer = [0u8; 4];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(i32::from_le_bytes(buffer)),
            IOEndian::Big => Ok(i32::from_be_bytes(buffer)),
        }
    }

    pub fn read_int64(&mut self) -> Result<i64, Box<dyn Error>> {
        let mut buffer = [0u8; 8];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(i64::from_le_bytes(buffer)),
            IOEndian::Big => Ok(i64::from_be_bytes(buffer)),
        }
    }

    // Read unsigned integers
    pub fn read_uint8(&mut self) -> Result<u8, Box<dyn Error>> {
        let mut buffer = [0u8; 1];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(u8::from_le_bytes(buffer)),
            IOEndian::Big => Ok(u8::from_be_bytes(buffer)),
        }
    }

    pub fn read_uint16(&mut self) -> Result<u16, Box<dyn Error>> {
        let mut buffer = [0u8; 2];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(u16::from_le_bytes(buffer)),
            IOEndian::Big => Ok(u16::from_be_bytes(buffer)),
        }
    }

    pub fn read_uint32(&mut self) -> Result<u32, Box<dyn Error>> {
        let mut buffer = [0u8; 4];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(u32::from_le_bytes(buffer)),
            IOEndian::Big => Ok(u32::from_be_bytes(buffer)),
        }
    }

    pub fn read_uint64(&mut self) -> Result<u64, Box<dyn Error>> {
        let mut buffer = [0u8; 8];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(u64::from_le_bytes(buffer)),
            IOEndian::Big => Ok(u64::from_be_bytes(buffer)),
        }
    }

    // Read floats
    pub fn read_float16(&mut self) -> Result<f16, Box<dyn Error>> {
        let mut buffer = [0u8; 2];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(f16::from_le_bytes(buffer)),
            IOEndian::Big => Ok(f16::from_be_bytes(buffer)),
        }
    }

    pub fn read_float32(&mut self) -> Result<f32, Box<dyn Error>> {
        let mut buffer = [0u8; 4];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(f32::from_le_bytes(buffer)),
            IOEndian::Big => Ok(f32::from_be_bytes(buffer)),
        }
    }

    pub fn read_float64(&mut self) -> Result<f64, Box<dyn Error>> {
        let mut buffer = [0u8; 8];
        self.read_bytes_into_slice(&mut buffer)?;

        match self.endian {
            IOEndian::Little => Ok(f64::from_le_bytes(buffer)),
            IOEndian::Big => Ok(f64::from_be_bytes(buffer)),
        }
    }

    // Read strings
    pub fn read_prefixed_string(&mut self) -> Result<String, Box<dyn Error>> {
        let length = self.read_int32()?;
        let raw_bytes = self.read_bytes(length as usize)?;

        // TODO: Replace with better one (FromUtf8Error message is awful)
        Ok(String::from_utf8(raw_bytes)?)
    }

    pub fn read_null_terminated_string(&mut self) -> Result<String, Box<dyn Error>> {
        let mut raw_bytes = Vec::new();

        loop {
            let b = self.read_uint8()?;
            if b == 0 {
                break
            }

            raw_bytes.push(b);
        }

        // TODO: Replace with better one (FromUtf8Error message is awful)
        Ok(String::from_utf8(raw_bytes)?)
    }
}

// Writer implementation
impl<'a> BinaryStream<'a> {
    // Write boolean
    pub fn write_boolean(&mut self, value: bool) -> Result<(), Box<dyn Error>> {
        let data = match value {
            true => 1u8,
            _ => 0u8,
        };

        self.write_uint8(data)
    }

    // Write signed integers
    pub fn write_int8(&mut self, value: i8) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    pub fn write_int16(&mut self, value: i16) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    pub fn write_int32(&mut self, value: i32) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    pub fn write_int64(&mut self, value: i64) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    // Write unsigned integers
    pub fn write_uint8(&mut self, value: u8) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    pub fn write_uint16(&mut self, value: u16) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    pub fn write_uint32(&mut self, value: u32) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    pub fn write_uint64(&mut self, value: u64) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    // Write floats
    pub fn write_float16(&mut self, value: f16) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    pub fn write_float32(&mut self, value: f32) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    pub fn write_float64(&mut self, value: f64) -> Result<(), Box<dyn Error>> {
        let data = match self.endian {
            IOEndian::Little => value.to_le_bytes(),
            IOEndian::Big => value.to_be_bytes(),
        };

        self.write_bytes(&data)
    }

    // Write strings
    pub fn write_prefixed_string(&mut self, value: &str) -> Result<(), Box<dyn Error>> {
        let data = value.as_bytes(); // Assumed to be utf-8

        self.write_int32(data.len() as i32)?;
        self.write_bytes(data)?;

        Ok(())
    }
}