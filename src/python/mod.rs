#![cfg(feature = "libloading")]
use crate::plugins::Plugins;

use pyo3::prelude::*;

#[pyclass]
pub struct LennaCli {
    plugins: Plugins,
}

impl LennaCli {
    pub fn load_plugins(&mut self, path: String) {
        let plugins_path = std::path::PathBuf::from(path);
        self.plugins.load_plugins(&plugins_path);
    }
}

impl Default for LennaCli {
    fn default() -> Self {
        LennaCli {
            plugins: Plugins::new(),
        }
    }
}
/*
#[pymodule]
pub fn lenna_cli(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<LennaCli>()?;

    Ok(())
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lenna_cli_class() {
        let mut lenna_cli_class = LennaCli::default();
        lenna_cli_class.load_plugins("plugins".to_string());
    }
}
