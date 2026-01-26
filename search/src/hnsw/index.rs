use std::{fs::OpenOptions, path::Path};

use sal_core::{dbg::Dbg, error::Error};
use sal_sync::collections::FxIndexMap;
use serde::Deserializer;

use crate::hnsw::Meta;

///
/// A collection of pairs `Index - Meta` for each embedded document
pub struct Index {
    pub path: String,
    pub index: FxIndexMap<usize, Meta>,
    dbg: Dbg,
}
//
impl Index {
    ///
    /// Returns [Index] new instance
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            index: FxIndexMap::default(),
            dbg: Dbg::own("Index"),
        }
    }
    ///
    /// Returns a Meta by the key if exists
    pub fn get(&self, key: usize) -> Option<&Meta> {
        self.index.get(&key)
    }
    ///
    /// Inserts a Meta  with generated key , updates a Meta if key already exists
    /// - returns a `key`
    pub fn insert(&mut self, v: Meta) -> usize {
        let k = self.index.len();
        self.index.insert(k, v);
        k
    }
    ///
    /// Stores all contined values on the disk
    pub fn store(&self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "store");
        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path);
        match f {
            Ok(f) => {
                serde_json::to_writer(f, &self.index)
                    .map_err(|err| error.pass_with(format!("Can't write to '{}'", self.path), err.to_string()))
            }
            Err(err) => Err(error.pass_with(format!("Can't open file '{}'", self.path), err.to_string())),
        }
    }
    pub fn load(path: impl Into<String>) -> Result<Self, Error> {
        let path = path.into();
        let error = Error::new("Index", "store");
        let f = OpenOptions::new()
            .read(true)
            .open(&path);
        match f {
            Ok(f) => {
                let index: FxIndexMap<usize, Meta> = serde_json::from_reader(f)
                   .map_err(|err| error.pass_with(format!("Can't read to '{}'", path), err.to_string()))?;
                Ok(Self {
                    path,
                    index,
                    dbg: Dbg::own("Index"),
                })
            }
            Err(err) => Err(error.pass_with(format!("Can't open file '{}'", path), err.to_string())),
        }
    }
}
