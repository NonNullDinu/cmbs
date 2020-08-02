//! # Stuff related to the compilers

// c compilers
mod cc;

// c++ compilers
mod cxx;

pub(crate) trait Compiler {
    fn can_consume(filename: &str) -> bool;
    fn can_compile(filename: &str) -> bool;
}
