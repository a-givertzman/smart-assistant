use std::{sync::Arc, time::Instant};

use hnsw_rs::{hnsw::Hnsw, prelude::DistCosine};
use sal_core::{dbg::Dbg, error::Error};

use crate::{domain::Eval, hnsw::{Index, Query, SearchHit}};

///
/// Transforms input text query into embedded vector
pub struct Search<'a> {
    knbn: usize,
    ef_search: usize,
    index: Arc<Index>,
    hnsw: Arc<Hnsw<'a, f32, DistCosine>>,
    ctx: Arc<Box<dyn Eval<String, Result<Query, Error>> + Send + Sync>>,
    dbg: Dbg,
}
//
impl<'a> Search<'a> {
    ///
    /// Returns [Search] new instance
    pub fn new(
        knbn: usize,
        ef_search: usize,
        index: Arc<Index>,
        hnsw: Arc<Hnsw<'a, f32, DistCosine>>,
        ctx: impl Eval<String, Result<Query, Error>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            knbn,
            ef_search,
            index,
            hnsw,
            ctx: Arc::new(Box::new(ctx)),
            dbg: Dbg::own("Search"),
        }
    }
}
//
impl<'a> Eval<String, Result<Vec<SearchHit>, Error>> for Search<'a> {
    fn eval(&self, query: String) -> Result<Vec<SearchHit>, Error> {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(query) {
            Ok(ctx) => {
                let query = ctx; 
                let t = Instant::now();
                let hits: Vec<SearchHit> = self.hnsw.search(&query.emb, self.knbn, self.ef_search)
                    .iter().filter_map(|h| {
                        match self.index.get(h.d_id) {
                            Some(meta) => Some(SearchHit { id: h.d_id, distance: h.distance, meta: meta.to_owned() }),
                            None => None,
                        }
                    }).collect();
                let elapsed = t.elapsed();
                log::debug!("{}.eval | Elapsed {:?}", self.dbg, elapsed);
                log::debug!("{}.eval | Search hits [{}]:", self.dbg, hits.len());
                for hit in &hits {
                    log::debug!("\t {:?}", hit);
                }
                Ok(hits)
            }
            Err(err) => Err(error.pass(err.to_string()))
        }
    }
}