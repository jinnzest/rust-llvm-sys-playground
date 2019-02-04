extern crate libc;

pub mod bignumloader;
pub mod generator;
pub mod llvm;

use generator::*;
use std::*;

fn main() {
    let argument = env::args().last().unwrap_or_default();
    if argument == "exec" {
        run_exec();
    } else if argument == "compile" {
        run_compile();
    } else {
        println!("expected one of exec or compile arguments");
    }
}

fn run_exec() {
    if llvm_exec() {
        std::process::exit(0)
    } else {
        std::process::exit(-1)
    }
}

fn run_compile() {
    if llvm_compile2("output") {
        std::process::exit(0)
    } else {
        std::process::exit(-1)
    }
}
