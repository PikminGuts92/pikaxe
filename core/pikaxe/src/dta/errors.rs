use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum ParseDTAError {
    #[error("Unknown DTA parse error")]
    UnknownDTAParseError,
}