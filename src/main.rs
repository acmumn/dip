#[macro_use]
extern crate structopt;
extern crate dip;
extern crate failure;

use std::path::PathBuf;

use failure::Error;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The root configuration directory for dip.
    #[structopt(short = "d", long = "root", parse(from_os_str))]
    root: PathBuf,
    /// A string containing the address to bind to.
    #[structopt(short = "b", long = "bind")]
    bind: Option<String>,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    println!("{:?}", opt);
    assert!(opt.root.exists());
    dip::run(opt.root)
}
