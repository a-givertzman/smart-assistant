use std::sync::Arc;

use model2vec_rs::model::StaticModel;
use sal_core::{dbg::Dbg, error::Error};

use crate::{domain::Eval, hnsw::Query};

///
/// Transforms input text query into embedded vector
pub struct EmbeddedQuery {
    model: Arc<StaticModel>,
    ctx: Arc<Box<dyn Eval<(), Result<(), Error>> + Send + Sync>>,
    dbg: Dbg,
}
//
impl EmbeddedQuery {
    ///
    /// Returns [EmbeddedQuery] new instance
    pub fn new(model: Arc<StaticModel>, ctx: impl Eval<(), Result<(), Error>> + Send + Sync + 'static,) -> Self {
        Self {
            model,
            ctx: Arc::new(Box::new(ctx)),
            dbg: Dbg::own("EmbeddedQuery"),
        }
    }
}
//
impl Eval<String, Result<Query, Error>> for EmbeddedQuery {
    fn eval(&self, query: String) -> Result<Query, Error> {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(()) {
            Ok(_) => {
                let query = query.trim();
                log::debug!("Query     {:?}", query);
                let query = self.model.encode_single(&query);
                log::debug!("Query embedding {:?}", query);
                Ok(Query { emb: query })
            }
            Err(err) => Err(error.pass(err.to_string()))
        }
    }
}