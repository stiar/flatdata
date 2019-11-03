#![allow(dead_code, missing_debug_implementations)]
// generate with from the root of the crate:
// ../../generator -g rust -s src/test_archive.flatdata -O src/test_archive_generated.rs
include!("test_archive_generated.rs");
pub use test::*;
