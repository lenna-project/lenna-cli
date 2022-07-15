use lenna_core::io::write::write_to_data;
use lenna_core::LennaImage;
use std::fs;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;

pub mod plugins;
mod wasm;

#[cfg(feature = "python")]
pub mod python;

#[cfg(not(target_arch = "wasm32"))]
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
        //let name = path.strip_prefix(Path::new(&entry.path)).unwrap();
        #[allow(deprecated)]
        zip.start_file_from_path(path, options)?;
        let buffer = write_to_data(entry, format.clone()).unwrap();

        zip.write_all(&*buffer)?;
    }
    zip.finish()?;
    Result::Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
pub fn write_to_path(mut img: Box<LennaImage>, path: String, ext: String) {
    let ext = ext.as_str();
    img.path = path.clone();
    match ext {
        "zip" => {
            img.name = format!("{}.jpg", img.name);
            let images = vec![&mut img];
            let file = std::fs::File::create(&path).unwrap();
            zip_images(
                images,
                image::ImageOutputFormat::Jpeg(80),
                file,
                zip::CompressionMethod::DEFLATE,
            )
            .unwrap();
        }
        "png" | "PNG" => {
            lenna_core::io::write::write_to_file(&img, image::ImageOutputFormat::Png).unwrap();
        }
        _ => {
            lenna_core::io::write::write_to_file(&img, image::ImageOutputFormat::Jpeg(80)).unwrap();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use lenna_core::io::read::read_from_file;

    #[test]
    fn test_images_in_path() {
        let path: PathBuf = [".", "lenna.png"].iter().collect();
        let images = images_in_path(&path);
        assert_eq!(images.len(), 1);
    }

    #[test]
    fn test_write_to_path() {
        let mut image: Box<LennaImage> = Box::new(read_from_file("lenna.png".into()).unwrap());
        image.name = "test".to_string();
        write_to_path(image, "./".to_string(), "png".to_string());
        let mut image: Box<LennaImage> = Box::new(read_from_file("lenna.png".into()).unwrap());
        image.name = "test".to_string();
        write_to_path(image, "./".to_string(), "jpg".to_string());
        let mut image: Box<LennaImage> = Box::new(read_from_file("lenna.png".into()).unwrap());
        image.name = "test".to_string();
        write_to_path(image, "./test.zip".to_string(), "zip".to_string());
        let png_metadata = fs::metadata("test.png").unwrap();
        let jpg_metadata = fs::metadata("test.jpg").unwrap();
        let zip_metadata = fs::metadata("test.zip").unwrap();
        assert!(png_metadata.len() > jpg_metadata.len());
        assert!(png_metadata.len() > zip_metadata.len());
    }
}
