use std::path::{Path, PathBuf};

#[derive(Debug, StructOpt)]
pub struct Config {
    /// The root configuration directory for dip. This argument is required.
    #[structopt(short = "d", long = "root", parse(from_os_str))]
    pub root: PathBuf,
    /// A string containing the address to bind to. This defaults to "0.0.0.0:5000".
    #[structopt(short = "b", long = "bind", default_value = "0.0.0.0:5000")]
    pub bind: String,
    /// If a hook is specified here, it will be triggered manually exactly once and then the
    /// program will exit rather than running as a server.
    #[structopt(short = "h", long = "hook")]
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
