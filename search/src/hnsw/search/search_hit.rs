use crate::hnsw::Meta;

///
/// Single result of the search
#[derive(Debug)]
pub struct SearchHit {
    /// identification of data vector as given in initializing hnsw
    pub id: usize,
    /// distance of neighbours
    pub distance: f32,
    /// Search result meta info
    pub meta: Meta,
}