use anyhow::{Context, Result};
use std::{fs::File, io::Read};

pub struct FileUtils {}
impl FileUtils {
    pub fn load(path: &str) -> Result<Vec<u8>> {
        let mut file = File::open(path).with_context(|| format!("Failed to open file: {}", path))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to read file: {}: {}", path, e))?;
        Ok(data)
    }
}
