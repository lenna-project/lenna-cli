#![cfg(feature = "libloading")]
use crate::plugins::Plugins;
use image::RgbImage;
use lenna_core::{LennaImage, Pipeline};
use ndarray::{Array3, ArrayView3};
use nshare::ToNdarray3;
use numpy::IntoPyArray;
use pyo3::prelude::*;

fn array_to_image(arr: Array3<u8>) -> RgbImage {
    assert!(arr.is_standard_layout());

    let (height, width, _) = arr.dim();
    let raw = arr.into_raw_vec();

    RgbImage::from_raw(width as u32, height as u32, raw)
        .expect("container should have the right size for the image dimensions")
}

pub fn py_process(pipeline: &Pipeline, data: ndarray::ArrayViewD<'_, u8>) -> ndarray::ArrayD<u8> {
    let image: ArrayView3<u8> = data.into_dimensionality::<ndarray::Ix3>().unwrap();

    let image: RgbImage = array_to_image(image.to_owned());
    let image: image::DynamicImage = image::DynamicImage::ImageRgb8(image);

    let mut lenna_image = Box::new(LennaImage::default());
    lenna_image.image = Box::new(image);

    pipeline.run(&mut lenna_image).unwrap();

    let image = lenna_image.image.to_rgb8();
    let image = image.into_ndarray3();
    // dimension is here (channel, row, col)
    let mut image = image.reversed_axes();
    image.swap_axes(0, 1);
    // dimension is here (row, col, channel)
    image.to_owned().into_dimensionality::<_>().unwrap()
}

#[pyclass]
pub struct LennaCli {
    plugins: Plugins,
}

#[pymethods]
impl LennaCli {
    #[new]
    fn new() -> Self {
        LennaCli::default()
    }

    pub fn load_plugins(&mut self, path: String) {
        let plugins_path = std::path::PathBuf::from(path);
        self.plugins.load_plugins(&plugins_path);
    }

    pub fn plugins(&self) -> Vec<String> {
        self.plugins.pool.ids()
    }

    pub fn process(
        &self,
        config: pyo3::PyObject,
        image: numpy::PyReadonlyArrayDyn<u8>,
    ) -> Py<numpy::PyArray<u8, numpy::Ix3>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let config = pythonize::depythonize(config.as_ref(py)).unwrap();

        let pipeline = Pipeline::new(config, self.plugins.pool.clone());

        let data = py_process(&pipeline, image.as_array());
        let image: Array3<u8> = data.into_dimensionality::<ndarray::Ix3>().unwrap();
        let image = image.to_owned();
        let data: &numpy::PyArray<_, _> = image.into_pyarray(py);
        data.to_owned()
    }
}

impl Default for LennaCli {
    fn default() -> Self {
        LennaCli {
            plugins: Plugins::new(),
        }
    }
}

#[pymodule]
pub fn lenna_cli(_py: pyo3::Python, m: &PyModule) -> pyo3::PyResult<()> {
    m.add_class::<LennaCli>()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lenna_cli_class() {
        let mut lenna_cli_class = LennaCli::default();
        lenna_cli_class.load_plugins("plugins".to_string());
    }
}
