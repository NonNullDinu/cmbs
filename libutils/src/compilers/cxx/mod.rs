use super::{Compiler, GetCompilerError};
use std::process::Command;

use std::path::PathBuf;

pub mod cxx_flags;
use crate::compilers::cxx::cxx_flags::CPPSTD;
pub use cxx_flags::{CXXFlag, CXXFlags, CXXLDFlag, CXXLDFlags};

#[derive(Copy, Clone)]
pub enum CXXFamily {
    GCC,
    Clang,
    MSVC,
}

#[derive(Clone)]
pub struct CXX {
    family: CXXFamily,
    location: PathBuf,
}

impl CXX {
    pub fn get_flag(&self, flag: CXXFlag) -> String {
        match self.family {
            CXXFamily::GCC | CXXFamily::Clang => match flag {
                CXXFlag::FromString { string } => string,
                CXXFlag::CPPSTD { std } => format!(
                    "--cpp_std={}",
                    match std {
                        CPPSTD::CPP98 => "c++98",
                        CPPSTD::CPP03 => "c++03",
                        CPPSTD::CPP1x => "c++1x",
                        CPPSTD::CPP1y => "c++1y",
                        CPPSTD::CPP1z => "c++1z",
                        CPPSTD::CPP2a => "c++2a",
                    }
                ),
                CXXFlag::IncludeDir { include_dir } => format!("-I {}", include_dir),
            },
            CXXFamily::MSVC => {
                // TODO add this later
                "".to_string()
            }
        }
    }
}

impl Compiler for CXX {
    fn can_consume(filename: &str) -> bool {
        Self::can_compile(filename)
            || filename.ends_with(".h")
            || filename.ends_with(".hpp")
            || filename.ends_with(".hh")
            || filename.ends_with(".hxx")
    }

    fn can_compile(filename: &str) -> bool {
        filename.ends_with(".cpp")
            || filename.ends_with(".cc")
            || filename.ends_with(".cxx")
            || filename.ends_with(".c")
    }

    fn get_location(&self) -> &PathBuf {
        &self.location
    }
}

pub fn get_cxx() -> Result<CXX, GetCompilerError> {
    let compiler_location = match std::env::var("CXX") {
        Ok(p) => Ok(PathBuf::from(p)),
        Err(err) => {
            if cfg!(target_os = "linux") {
                Ok(PathBuf::from("/usr/bin/c++"))
            } else {
                Err(err)
            }
        }
    }?;

    let location = compiler_location.clone();

    let output = Command::new(compiler_location).arg("--version").output()?;
    let output = String::from_utf8(output.stdout)?;
    let first_line = output
        .lines()
        .next() // get first line
        .expect("Cannot detect compiler family from `CXX --version'");

    match first_line {
        family if family.contains("(GCC)") => Ok(CXX {
            family: CXXFamily::GCC,
            location,
        }),
        family if family.contains("clang") => Ok(CXX {
            family: CXXFamily::Clang,
            location,
        }),
        family => Err(GetCompilerError::UnrecognizedCompilerFamily(
            family.to_string(),
        )),
    }
}
