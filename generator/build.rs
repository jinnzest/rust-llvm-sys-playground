fn main() {
    println!("cargo:rustc-link-search=/Users/jinnzest/Documents/nulljinn/tests/libtommath/");
    println!("cargo:rustc-link-lib=static=tommath");
}
