#[cfg(feature = "python")] use pyo3::prelude::*;
use std::{path::PathBuf, todo};

#[derive(Debug, Default)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Ark {
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub version: i32,
    pub encryption: ArkEncryption,
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub entries: Vec<ArkOffsetEntry>,
    pub path: PathBuf, // Hdr/ark path,
    pub part_paths: Vec<PathBuf>,
}

#[derive(Debug)]
//#[cfg_attr(feature = "python", pyclass)]
pub enum ArkEncryption {
    None,
    ClassicEncryption(i32),
    NewEncryption(i32),
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyclass)]
pub struct ArkOffsetEntry {
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub id: u32,
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub path: String,
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub offset: u64,
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub part: u32,
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub size: usize,
    #[cfg_attr(feature = "pyo3", pyo3(get, set))] pub inflated_size: usize
}

impl ArkOffsetEntry {
    pub fn is_gen_file(&self) -> bool {
        if !self.path.contains('/') {
            return false;
        }

        // Check last directory name for "gen" (there's gotta be a cleaner way to do this)
        self.path
            .split("/")
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .skip(1) // Skip file name
            .next()
            .map(|d| d.eq_ignore_ascii_case("gen"))
            .unwrap_or_default()
    }
}

impl Default for ArkEncryption {
    fn default() -> Self {
        ArkEncryption::None
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Ark {
    #[staticmethod]
    fn from_file_path(path: &str) -> PyResult<Ark> {
        // TODO: Convert to PyResult
        let ark = Ark::from_path(path).unwrap();
        Ok(ark)
    }

    #[getter]
    fn get_encryption(&self) -> PyResult<Option<i32>> {
        let key = match self.encryption {
            ArkEncryption::ClassicEncryption(key) => Some(key),
            ArkEncryption::NewEncryption(key) => Some(key),
            _ => None
        };

        Ok(key)
    }
}

impl Ark {
    pub fn get_stream(&self, id: u32) -> Result<Vec<u8>, std::io::Error> {
        use std::io::{Read, Seek, SeekFrom};

        let entry = self
            .entries
            .iter()
            .find(|e| e.id == id)
            .expect("Invalid id");

        // Open from main ark or ark part
        let file_path = if self.version >= 3 && self.version <= 10 {
            &self.part_paths[entry.part as usize]
        } else {
            &self.path
        };

        // TODO: Support reading from non-first ark part?
        let mut file = std::fs::File::open(file_path)?;
        file.seek(SeekFrom::Start(entry.offset))?;

        let mut buffer = vec![0u8; entry.size];
        file.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}