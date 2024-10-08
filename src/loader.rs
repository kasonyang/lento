use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use anyhow::anyhow;
use quick_js::loader::{FsJsModuleLoader, JsModuleLoader};

pub struct RemoteModuleLoader {

}

impl RemoteModuleLoader {
    pub fn new() -> Self {
        Self {}
    }
}

impl JsModuleLoader for RemoteModuleLoader {
    fn load(&self, module_name: &str) -> Result<String, Error> {
        let url = if module_name.starts_with("http://") || module_name.starts_with("https://") {
            module_name.to_string()
        } else {
            return Err(Error::new(ErrorKind::AddrNotAvailable, anyhow!("Failed to resolve module: {}", module_name)));
        };
        let body = reqwest::blocking::get(&url).map_err(|e| Error::new(ErrorKind::Other, e))?
            .text().map_err(|e| Error::new(ErrorKind::Other, e))?;
        Ok(body)
    }
}

pub struct StaticModuleLoader {
    sources: HashMap<String, String>,
}

impl StaticModuleLoader {
    pub fn new() -> Self {
        StaticModuleLoader {
            sources: HashMap::new(),
        }
    }
    pub fn add_module(&mut self, module_name: String, source: String) {
        self.sources.insert(module_name, source);
    }
}

impl JsModuleLoader for StaticModuleLoader {
    fn load(&self, module_name: &str) -> Result<String, Error> {
        match self.sources.get(module_name) {
            None => Err(Error::new(ErrorKind::NotFound, anyhow!("Not found"))),
            Some(s) => Ok(s.to_string())
        }
    }
}

pub struct DefaultModuleLoader {
    remote_module_loader: Option<RemoteModuleLoader>,
    fs_module_loader: Option<FsJsModuleLoader>,
}

impl DefaultModuleLoader {

    pub fn new(allow_remote: bool) -> Self {
        let remote_module_loader = if allow_remote {
            Some(RemoteModuleLoader::new())
        } else {
            None
        };
        Self {
            remote_module_loader,
            fs_module_loader: None,
        }
    }

    pub fn set_fs_base(&mut self, dir: &str) {
        self.fs_module_loader = Some(FsJsModuleLoader::new(dir))
    }

}

impl JsModuleLoader for DefaultModuleLoader {
    fn load(&self, module_name: &str) -> Result<String, Error> {
        if let Some(fs_loader) = &self.fs_module_loader {
            if let Ok(module) = fs_loader.load(module_name) {
               return Ok(module)
            }
        }
        if let Some(remote_loader) = &self.remote_module_loader {
            return remote_loader.load(module_name);
        }
        Err(Error::new(ErrorKind::NotFound, "failed to load module"))
    }
}