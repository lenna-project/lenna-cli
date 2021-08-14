#![cfg(not(target_arch = "wasm32"))]
#![cfg(feature = "libloading")]
use lenna_core::plugins::PluginDeclaration;
use lenna_core::{ExifProcessor, ImageProcessor, Pool, Processor, ProcessorConfig};
use libloading::Library;
use std::{ffi::OsStr, fs, io, path::PathBuf, rc::Rc};

#[derive(Clone)]
pub struct PluginProxy {
    processor: Box<dyn Processor>,
    _lib: Rc<Library>,
}

impl ImageProcessor for PluginProxy {}
impl ExifProcessor for PluginProxy {}

impl Processor for PluginProxy {
    fn id(&self) -> String {
        self.processor.id()
    }
    fn name(&self) -> String {
        self.processor.name()
    }
    fn title(&self) -> String {
        self.processor.title()
    }
    fn version(&self) -> String {
        self.processor.version()
    }
    fn author(&self) -> String {
        self.processor.author()
    }
    fn description(&self) -> String {
        self.processor.description()
    }

    fn process(
        &mut self,
        config: ProcessorConfig,
        image: &mut Box<lenna_core::LennaImage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.processor.process(config, image)
    }

    fn set_config(&mut self, config: serde_json::Value) {
        self.processor.set_config(config)
    }
    fn default_config(&self) -> serde_json::Value {
        self.processor.default_config()
    }
    fn config_ui(&self) -> Option<String> {
        self.processor.config_ui()
    }
    fn icon(&self) -> Option<Vec<u8>> {
        self.processor.icon()
    }
}

struct PluginRegistrar {
    plugins: Vec<PluginProxy>,
    lib: Rc<Library>,
}

impl PluginRegistrar {
    fn new(lib: Rc<Library>) -> PluginRegistrar {
        PluginRegistrar {
            lib,
            plugins: Vec::default(),
        }
    }
}

impl lenna_core::plugins::PluginRegistrar for PluginRegistrar {
    fn add_plugin(&mut self, processor: Box<dyn Processor>) {
        let proxy = PluginProxy {
            processor,
            _lib: Rc::clone(&self.lib),
        };
        self.plugins.push(proxy);
    }
}

#[derive(Default)]
pub struct Plugins {
    pub pool: Pool,
    libraries: Vec<Rc<Library>>,
}

impl Plugins {
    pub fn new() -> Plugins {
        Plugins::default()
    }

    pub fn load_plugins(&mut self, plugins_path: &PathBuf) {
        let extensions = ["so", "dll", "dylib"];
        let paths = fs::read_dir(plugins_path);
        match paths {
            Ok(paths) => {
                for library_path in paths {
                    match library_path {
                        Ok(path) => {
                            let file = path.path();
                            let extension = &file.extension();
                            match extension {
                                Some(extension) => {
                                    if extensions.contains(&extension.to_str().unwrap()) {
                                        unsafe {
                                            match self.load(file) {
                                                Ok(_) => (),
                                                Err(e) => println!("{:?}", e),
                                            }
                                        }
                                    }
                                }
                                _ => (),
                            }
                        }
                        Err(err) => println!("{:?}", err),
                    }
                }
            }
            Err(err) => println!("{:?}", err),
        };
    }

    pub unsafe fn load<P: AsRef<OsStr>>(&mut self, library_path: P) -> io::Result<()> {
        // load the library into memory
        let library = Rc::new(Library::new(library_path).unwrap());

        // get a pointer to the plugin_declaration symbol.
        let decl = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")
            .unwrap()
            .read();

        // version checks to prevent accidental ABI incompatibilities
        if decl.rustc_version != lenna_core::RUSTC_VERSION
            || decl.core_version != lenna_core::CORE_VERSION
        {
            return Err(io::Error::new(io::ErrorKind::Other, "Version mismatch"));
        }

        let mut registrar = PluginRegistrar::new(Rc::clone(&library));

        (decl.register)(&mut registrar);

        let plugin = registrar.plugins.swap_remove(0);
        self.pool.add(Box::new(plugin));

        //self.plugins.extend(registrar.plugins);
        self.libraries.push(library);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let plugins = Plugins::new();
        assert_eq!(plugins.pool.ids().len(), 1);
    }

    #[test]
    fn load_path() {
        let mut plugins = Plugins::new();
        let path = std::path::PathBuf::from("plugins/");
        plugins.load_plugins(&path);
        assert_eq!(plugins.pool.ids().len(), 1);
    }

    #[test]
    fn load_non_existend_path() {
        let mut plugins = Plugins::new();
        let path = std::path::PathBuf::from("pluginss/");
        plugins.load_plugins(&path);
        assert_eq!(plugins.pool.ids().len(), 1);
    }

    #[test]
    fn proxy() {
        let plugins = Plugins::new();
        let processor = plugins.pool.get("resize").unwrap();
        unsafe {
            let library = Rc::new(Library::new("").unwrap());

            let proxy = PluginProxy {
                processor,
                _lib: library,
            };

            assert!(proxy.icon().is_some());
            assert_eq!(proxy.icon().unwrap().len(), 36408);
        }
    }
}
