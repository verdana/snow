use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Component, Path, PathBuf},
};

use anyhow::{bail, Ok};

pub(crate) fn relative(path1: &str, path2: &str) -> anyhow::Result<String> {
    let abs_path1 = Path::new(path1);
    let abs_path2 = Path::new(path2);
    if !abs_path1.is_absolute() || !abs_path2.is_absolute() {
        bail!("Both path1 and path2 must be absolute paths".to_string())
    }

    let common = common_prefix(path1, path2);
    if common.is_empty() || (common.len() == 1 && common[0] == Component::RootDir) {
        return Ok(path1.to_string());
    }

    let mut base1 = abs_path1
        .components()
        .skip(common.len())
        .collect::<PathBuf>();

    let mut base2 = abs_path2
        .components()
        .skip(common.len())
        .collect::<PathBuf>();
    base2.pop();

    if base2 == Path::new(".") {
        return Ok(base1.to_string_lossy().into_owned());
    }
    for _ in 0..base2.components().count() {
        base1 = Path::new("..").join(base1);
    }

    Ok(base1.to_string_lossy().into_owned())
}

/// Find the common prefix of two paths
pub(crate) fn common_prefix<'a>(path1: &'a str, path2: &'a str) -> Vec<Component<'a>> {
    let components1 = Path::new(path1).components();
    let components2 = Path::new(path2).components();

    let mut common = Vec::new();
    for (c1, c2) in components1.zip(components2) {
        if c1 == c2 {
            common.push(c1);
        } else {
            break;
        }
    }
    common
}

/// Check if two files are the same
pub(crate) fn is_same_file(file1: &Path, file2: &Path) -> anyhow::Result<bool> {
    let metadata1 = fs::metadata(file1)?;
    let metadata2 = fs::metadata(file2)?;

    // Compare the inode and device ID of two files
    let result = metadata1.ino() == metadata2.ino() && metadata1.dev() == metadata2.dev();
    Ok(result)
}

/// Get the real path of the symlink and join it with the directory path of the symlink
pub fn get_symlink_real_path(link: &str) -> anyhow::Result<PathBuf> {
    let filepath = Path::new(link);
    let realpath = fs::read_link(filepath)?;

    // Get the directory path of the link
    let link_dir = filepath.parent().ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to get directory of the link",
    ))?;
    Ok(link_dir.join(realpath.clone()))
}
