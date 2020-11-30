use crate::diagnostics::{DiagConfig, FileId, LeafDiagnostic, LeafDiagnosticTrait, LeafLabel};
use crate::grammar::lexer::Token;
use crate::grammar::GrmError;
use lalrpop_util::ParseError;
macro_rules! error_codes {
    ($([$name:ident, $file:expr]),* $(,)?) => {
        error_codes!(1, $([$name, $file])*);
    };
    ($start:expr, [$first_name:ident, $first_file:literal] $(,)?) => {
        const $first_name: usize = $start;
        include!(concat!("errors/", $first_file));
    };
    ($start:expr, [$first_name:ident, $first_file:literal], $([$other_name:ident, $other_file:literal]),* $(,)?) => {
        const $first_name: usize = $start;
        include!(concat!("errors/", $first_file));
        error_codes!($start+1, $([$other_name,$other_file])*);
    };
}

error_codes! {[PARSE_ERROR, "parse_error.rs"],}
