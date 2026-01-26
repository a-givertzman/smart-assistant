use std::{sync::Arc, time::Instant};

use sal_core::{dbg::Dbg, error::Error};

use crate::{domain::Eval, hnsw::SearchHit};

///
/// Transforms search hits into a text ready for the LLM Model
pub struct ModelQuery<'a> {
    ctx: Arc<Box<dyn Eval<String, Result<Vec<SearchHit>, Error>> + Send + Sync + 'a>>,
    dbg: Dbg,
}
//
impl<'a> ModelQuery<'a> {
    ///
    /// Returns [ModelQuery] new instance
    pub fn new(
        ctx: impl Eval<String, Result<Vec<SearchHit>, Error>> + Send + Sync + 'a,
    ) -> Self {
        Self {
            ctx: Arc::new(Box::new(ctx)),
            dbg: Dbg::own("ModelQuery"),
        }
    }
}
//
impl<'a> Eval<String, Result<String, Error>> for ModelQuery<'a> {
    fn eval(&self, query: String) -> Result<String, Error> {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(query) {
            Ok(ctx) => {
                let hits = ctx;
                let t = Instant::now();
                // 
                let hits = hits.iter().fold(String::new(), |acc, hit| {
                    match std::fs::read_to_string(&hit.meta.path) {
                        Ok(text) => {
                            format!("{acc}\n{text}")
                        }
                        Err(err) => {
                            log::warn!("{}.eval | Can't read '{}', \n\terror: {:?}", self.dbg, hit.meta.path, err);
                            String::new()
                        }
                    }
                });
                let elapsed = t.elapsed();
                log::debug!("{}.eval | Elapsed {:?}", self.dbg, elapsed);
                Ok(hits)
            }
            Err(err) => Err(error.pass(err.to_string()))
        }
    }
}