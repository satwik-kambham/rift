use std::{
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path,
};

/// Read file at path to string
pub fn read_file_content(path: &str) -> Result<String, Box<dyn Error>> {
    let mut f = File::open(path)?;
    let mut buf = String::new();

    let _ = f.read_to_string(&mut buf)?;

    Ok(buf)
}

/// Override file at path with new content
pub fn override_file_content(path: &str, buf: String) -> Result<(), Box<dyn Error>> {
    let mut f = File::create(path)?;
    f.write_all(buf.as_bytes())?;

    Ok(())
}

/// Create directory at path (recursively)
pub fn create_directory(path: &str) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Create file at path
pub fn create_file(path: &str) -> Result<(), Box<dyn Error>> {
    override_file_content(path, "".into())?;
    Ok(())
}

/// Delete file at path
pub fn delete_file(path: &str) -> Result<(), Box<dyn Error>> {
    fs::remove_file(path)?;
    Ok(())
}

/// Delete directory with all its contents
pub fn delete_directory_recursively(path: &str) -> Result<(), Box<dyn Error>> {
    fs::remove_dir_all(path)?;
    Ok(())
}

/// Rename file or directory
pub fn rename_file_or_directory(path: &str, to: &str) -> Result<(), Box<dyn Error>> {
    let mut new_path = path::PathBuf::from(path);
    new_path.pop();
    new_path.push(to);
    fs::rename(path, new_path)?;
    Ok(())
}

/// Duplicate file and append _copy to new file
pub fn duplicate_file(path: &str) -> Result<(), Box<dyn Error>> {
    let mut new_path = path::PathBuf::from(path);
    if let Some(stem) = new_path.file_stem() {
        if let Some(extension) = new_path.extension() {
            new_path.set_file_name(format!(
                "{}_copy.{}",
                stem.to_string_lossy(),
                extension.to_string_lossy()
            ));
        } else {
            new_path.set_file_name(format!("{}_copy", stem.to_string_lossy()));
        }
    }
    Ok(())
}

/// Move file or directory to new path
pub fn move_file_or_directory(path: &str, to: &str) -> Result<(), Box<dyn Error>> {
    fs::rename(path, to)?;
    Ok(())
}

/// TODO: Get all items in folder
pub fn get_directory_entries(path: &str) -> Result<(), Box<dyn Error>> {
    Ok(())
}
