#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![allow(incomplete_features)]
#![feature(unsize)]
#![feature(specialization)]
#![allow(unused)]
#![deny(warnings)]

use clap::Parser;

mod args;
mod change_tracker;
mod query;
mod server_world;
fn main() {
    let args = args::Args::parse();
}
