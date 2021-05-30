#![cfg(target_arch = "wasm32")]
use image;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = convert)]
pub fn convert(data: &[u8], format: String) -> Vec<u8> {
    use console_error_panic_hook;
    console_error_panic_hook::set_once();

    let img = lenna_core::io::read::read_from_data(data.to_vec()).unwrap();
    let format: &str = &format.to_string();
    let format: image::ImageOutputFormat = match format {
        "jpg" => image::ImageOutputFormat::Jpeg(90),
        "png" => image::ImageOutputFormat::Png,
        _ => image::ImageOutputFormat::Jpeg(90),
    };
    let out_data = lenna_core::io::write::write_to_data(&img, format).unwrap();

    out_data
}
