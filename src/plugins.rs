use lenna_core::plugins::PluginDeclaration;
use lenna_core::Pool;
use libloading::Library;
use std::{ffi::OsStr, fs, io, path::PathBuf, rc::Rc};

#[derive(Default)]
pub struct Plugins {
    libraries: Vec<Rc<Library>>,
}

impl Plugins {
    pub fn new() -> Plugins {
        Plugins::default()
    }

    pub fn load_plugins(&mut self, pool: &mut Pool, plugins_path: &PathBuf) {
        let extensions = ["so", "dll", "dylib"];
        let paths = fs::read_dir(plugins_path).unwrap();

        for library_path in paths {
            let file = library_path.unwrap().path();
            let extension = &file.extension();
            if extensions.contains(&extension.unwrap().to_str().unwrap()) {
                unsafe {
                    match self.load(pool, file) {
                        Ok(_) => (),
                        Err(e) => println!("{:?}", e),
                    }
                }
            }
        }
    }

    pub unsafe fn load<P: AsRef<OsStr>>(
        &mut self,
        pool: &mut Pool,
        library_path: P,
    ) -> io::Result<()> {
        // load the library into memory
        let library = Rc::new(Library::new(library_path)?);

        // get a pointer to the plugin_declaration symbol.
        let decl = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")?
            .read();

        // version checks to prevent accidental ABI incompatibilities
        if decl.rustc_version != lenna_core::RUSTC_VERSION
            || decl.core_version != lenna_core::CORE_VERSION
        {
            return Err(io::Error::new(io::ErrorKind::Other, "Version mismatch"));
        }

        (decl.register)(pool);
        self.libraries.push(library);
        Ok(())
    }
}
