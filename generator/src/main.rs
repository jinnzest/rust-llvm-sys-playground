extern crate libc;

pub mod generator;
pub mod llvm;

use generator::*;

fn main() {
    if llvm_compile2() {
        std::process::exit(0)
    } else {
        std::process::exit(-1)
    }
}
