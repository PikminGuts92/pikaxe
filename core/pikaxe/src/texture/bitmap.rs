use crate::{Platform, SystemInfo};
use crate::io::IOEndian;
#[cfg(feature = "python")] use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Bitmap {
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub bpp: u8,
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub encoding: u32,
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub mip_maps: u8,

    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub width: u16,
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub height: u16,
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub bpl: u16,

    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub raw_data: Vec<u8>,
}

impl Bitmap {
    pub fn new() -> Bitmap {
        Bitmap {
            bpp: 8,
            encoding: 3,
            mip_maps: 0,

            width: 0,
            height: 0,
            bpl: 0,

            raw_data: Vec::new()
        }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Bitmap {
    #[staticmethod]
    fn from_file_path(path: &str) -> PyResult<Bitmap> {
        // TODO: Get from args
        let sys_info = SystemInfo {
            platform: Platform::PS3,
            ..SystemInfo::default()
        };

        let image_input = super::Image::FromPath(path.to_owned());
        let bitmap = Bitmap::from_image(image_input, &sys_info);

        Ok(bitmap)
    }

    fn save_to_file(&self, path: &str) -> PyResult<()> {
        use crate::io::{BinaryStream, FileStream, Stream};
        use crate::scene::ObjectReadWrite;
        use std::path::Path;

        // TODO: Get from args
        let sys_info = SystemInfo {
            platform: Platform::PS3,
            endian: IOEndian::Little,
            ..SystemInfo::default()
        };

        // Ugh so much damn boilerplate...
        let file_path = Path::new(path);
        crate::io::create_missing_dirs(file_path).unwrap();

        let mut file_stream = FileStream::from_path_as_read_write_create(file_path).unwrap();
        self.save(&mut file_stream, &sys_info).unwrap();

        Ok(())
    }
}