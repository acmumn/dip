extern crate dip;
extern crate failure;
extern crate structopt;

use dip::Config;
use failure::Error;
use structopt::StructOpt;

fn main() -> Result<(), Error> {
    let config = Config::from_args();
    dip::run(&config)
}
