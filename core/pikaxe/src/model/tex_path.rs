use std::path::PathBuf;

#[derive(Debug)]
pub struct TexPath {
    pub name: String,
    pub rgba: Vec<u8>,
    pub png_path: PathBuf
}