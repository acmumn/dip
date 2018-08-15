use std::path::{Path, PathBuf};

pub struct Config {
    pub root: PathBuf,
    pub bind: String,
    pub hook: Option<String>,
}

impl Config {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        assert!(root.exists());

        let bind = "0.0.0.0:5000".to_owned();
        let hook = None;
        Config { root, bind, hook }
    }
    pub fn bind(mut self, value: Option<String>) -> Config {
        if let Some(value) = value {
            self.bind = value;
        }
        return self;
    }
    pub fn hook(mut self, value: Option<String>) -> Config {
        self.hook = value;
        return self;
    }
}
