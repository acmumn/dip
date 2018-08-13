#[macro_use]
extern crate structopt;
extern crate dip;
extern crate failure;

use std::path::PathBuf;

use failure::Error;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "d", long = "root", parse(from_os_str))]
    root: PathBuf,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    println!("{:?}", opt);
    assert!(opt.root.exists());
    dip::run(opt.root)
}
