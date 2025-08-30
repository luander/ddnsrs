use std::error::Error;
use std::fs;
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
    let contents = fs::read_to_string(&path)?.trim().into();
    Ok(Some(contents))
}

pub fn write_cached_ip(ip: &str) -> Result<(), Box<dyn Error>> {
    let path = get_cache_path();
    fs::write(&path, ip)?;
    info!("Cached IP {} to file {:?}", ip, &path);
    Ok(())
}
