use serde::{Deserialize, Serialize};

///
/// Contains database fields about document
#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub path: String,
}
//
impl Meta {
    ///
    /// Returns [Meta] new instance
    /// - `name`: The name of the document
    /// - `section`: Section Tag index inside a document 
    /// - `page`: Page tag index
    /// - `path`: Local path to the document file
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
        }
    }
}
