use lenna_core::io::write::write_to_data;
use std::fs;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;

pub mod plugins;
mod wasm;

pub fn zip_images<T>(
    images: Vec<&mut Box<lenna_core::LennaImage>>,
    format: image::ImageOutputFormat,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    for entry in images {
        let path = Path::new(&entry.name);
        let name = path.strip_prefix(Path::new(&entry.path)).unwrap();
        #[allow(deprecated)]
        zip.start_file_from_path(name, options)?;
        let buffer = write_to_data(entry, format.clone()).unwrap();

        zip.write_all(&*buffer)?;
    }
    zip.finish()?;
    Result::Ok(())
}

pub fn images_in_path(path: &PathBuf) -> Vec<PathBuf> {
    let mut images: Vec<PathBuf> = Vec::new();
    let path = Path::new(path);
    match path.is_dir() {
        true => {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                } else {
                    images.push(path);
                }
            }
        }
        false => images.push(path.into()),
    }
    images
}
