use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use tracing::info;

fn get_cache_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("ddnsrs.ip");
    path
}

pub fn read_cached_ip() -> Result<Option<String>, Box<dyn Error>> {
    let path = get_cache_path();
    if !path.exists() {
        return Ok(None);
    }
    info!("Cache path: {:?}", path);
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(Some(contents.trim().to_string()))
}

pub fn write_cached_ip(ip: &str) -> Result<(), Box<dyn Error>> {
    let path = get_cache_path();
    let mut file = fs::File::create(&path)?;
    file.write_all(ip.as_bytes())?;
    info!("Cached IP {} to file {:?}", ip, &path);
    Ok(())
}
