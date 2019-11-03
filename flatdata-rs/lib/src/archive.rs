//! This module contains traits that are used by generated code to define
//! flatdata's archives.
//!
//! # Archive
//!
//! A flatdata archive consists of two types: read-only archive `ArchiveName`
//! which implements the trait `Archive` and provides access to resources, and
//! an archive builder `ArchiveNameBuilder` which implements `ArchiveBuilder`
//! trait. Both types are generated from a flatdata schema.
use crate::{error::ResourceStorageError, storage::ResourceStorage};

use std::{fmt::Debug, rc::Rc};

#[doc(hidden)]
pub use std::marker;

/// A flatdata archive representing serialized data.
///
/// Each archive in generated code implements this trait.
pub trait Archive: Debug + Clone {
    /// Name of the archive.
    const NAME: &'static str;
    /// Schema of the archive.
    ///
    /// Used for verifying the integrity of the archive when opening.
    const SCHEMA: &'static str;

    /// Opens the archive with name `NAME` and schema `SCHEMA` in the given
    /// storage for reading.
    ///
    /// When opening the archive, the schema of the archive and the schema
    /// stored in the storage are compared as strings. If there is a
    /// difference, an Error [`ResourceStorageError::WrongSignature`](enum.
    /// ResourceStorageError.html) is returned containing a detailed diff
    /// of both schemata.
    ///
    /// All resources are in the archive are also opened and their schemata are
    /// verified. If any non-optional resource is missing or has a wrong
    /// signature (unexpected schema), the operation will fail. Therefore,
    /// it is not possible to open partially written archive.
    fn open(storage: Rc<dyn ResourceStorage>) -> Result<Self, ResourceStorageError>;
}

/// A flatdata archive builder for serializing data.
///
/// For each archive in generated code there is a corresponding archive builder
/// which implements this trait.
pub trait ArchiveBuilder: Clone {
    /// Name of the archive associated with this archive builder.
    const NAME: &'static str;
    /// Schema of the archive associated with this archive builder.
    ///
    /// Used only for debug and inspection purposes.
    const SCHEMA: &'static str;

    /// Creates an archive with name `NAME` and schema `SCHEMA` in the given
    /// storage for writing.
    ///
    /// If the archive is successfully created, the storage will contain the
    /// archive and archives schema. Archive's resources need to be written
    /// separately by using the corresponding generated methods:
    ///
    /// * `set_struct`
    /// * `set_vector`
    /// * `start_vector`/`finish_vector`
    /// * `start_multivector`/`finish_multivector`.
    ///
    /// For more information about how to write resources, cf. the
    /// [coappearances] example.
    ///
    /// [coappearances]: https://github.com/boxdot/flatdata-rs/blob/master/tests/coappearances_test.rs#L159
    fn new(storage: Rc<dyn ResourceStorage>) -> Result<Self, ResourceStorageError>;
}
