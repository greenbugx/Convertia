use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionError {
    InputNotFound(String),
    InvalidInputPath(String),
    UnsupportedInputFormat(String),
    UnsupportedOutputFormat(String),
    DecodeFailed(String),
    EncodeFailed(String),
    InvalidTransform(String),
    InvalidOptions(String),
    OutputPath(String),
    FileSystem(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputNotFound(path) => write!(f, "Input file not found: {path}"),
            Self::InvalidInputPath(detail) => write!(f, "Invalid input path: {detail}"),
            Self::UnsupportedInputFormat(detail) => {
                write!(f, "Unsupported input format: {detail}")
            }
            Self::UnsupportedOutputFormat(detail) => {
                write!(f, "Unsupported output format: {detail}")
            }
            Self::DecodeFailed(detail) => write!(f, "Could not decode the input image: {detail}"),
            Self::EncodeFailed(detail) => write!(f, "Could not encode the output image: {detail}"),
            Self::InvalidTransform(detail) => write!(f, "Invalid transformation: {detail}"),
            Self::InvalidOptions(detail) => write!(f, "Invalid conversion options: {detail}"),
            Self::OutputPath(detail) => write!(f, "Invalid output path: {detail}"),
            Self::FileSystem(detail) => write!(f, "File system error: {detail}"),
        }
    }
}

impl std::error::Error for ConversionError {}
