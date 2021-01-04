#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/leafbuild/leafbuild/master/leaf_icon.svg",
    html_logo_url = "https://raw.githubusercontent.com/leafbuild/leafbuild/master/leaf_icon.svg"
)]
#![forbid(
    unsafe_code,
    unused_allocation,
    coherence_leak_check,
    confusable_idents,
    trivial_bounds
)]
#![deny(
    missing_docs,
    missing_crate_level_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    unused_imports,
    unused_import_braces,
    deprecated,
    broken_intra_doc_links,
    where_clauses_object_safety,
    order_dependent_trait_objects,
    unconditional_panic,
    unconditional_recursion,
    indirect_structural_match
)]
#![deny(
    clippy::correctness,
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::nursery
)]
#![allow(clippy::module_name_repetitions)]
//! # leafbuild-core
//! A crate that exposes some core structures and traits.
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate thiserror;

pub mod diagnostics;
pub mod lf_buildsys;
pub mod utils;

pub mod prelude {
    //! The prelude
    pub use super::diagnostics::{DiagCtx, LeafDiagnosticTrait};
    pub use super::lf_buildsys::LfBuildsys;
}
