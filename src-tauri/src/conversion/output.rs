use std::fs;
use std::path::Path;

use super::error::ConversionError;

pub fn validate_path(path: &Path) -> Result<(), ConversionError> {
    if path.as_os_str().is_empty() {
        return Err(ConversionError::OutputPath(
            "output path is empty".to_string(),
        ));
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.is_dir() {
            return Err(ConversionError::OutputPath(format!(
                "output directory does not exist: {}",
                parent.display()
            )));
        }
    }
    Ok(())
}

pub fn write_file(path: &Path, data: &[u8]) -> Result<(), ConversionError> {
    validate_path(path)?;
    fs::write(path, data).map_err(|error| {
        ConversionError::OutputPath(format!(
            "could not write output file {}: {error}",
            path.display()
        ))
    })
}
