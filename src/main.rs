#[macro_use]
extern crate structopt;
extern crate dip;
extern crate failure;

use std::path::PathBuf;

use dip::Config;
use failure::Error;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The root configuration directory for dip. This argument is required.
    #[structopt(short = "d", long = "root", parse(from_os_str))]
    root: PathBuf,
    /// A string containing the address to bind to. This defaults to "0.0.0.0:5000".
    #[structopt(short = "b", long = "bind")]
    bind: Option<String>,
    /// If a hook is specified here, it will be triggered manually exactly once and then the
    /// program will exit rather than running as a server.
    #[structopt(short = "h", long = "hook")]
    hook: Option<String>,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let config = Config::new(opt.root).bind(opt.bind).hook(opt.hook);
    dip::run(&config)
}
