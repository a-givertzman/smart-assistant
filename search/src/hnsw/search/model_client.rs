use std::{net::TcpStream, sync::Arc, time::Instant};

use sal_core::{dbg::Dbg, error::Error};

use crate::domain::Eval;

///
/// Sending a query to the model, returns model answer
pub struct ModelClient<'a> {
    model_addr: String,
    ctx: Arc<Box<dyn Eval<String, Result<String, Error>> + Send + Sync + 'a>>,
    dbg: Dbg,
}
//
impl<'a> ModelClient<'a> {
    ///
    /// Returns [ModelClient] new instance
    pub fn new(
        model_addr: impl Into<String>,
        ctx: impl Eval<String, Result<String, Error>> + Send + Sync + 'a,
    ) -> Self {
        Self {
            model_addr: model_addr.into(),
            ctx: Arc::new(Box::new(ctx)),
            dbg: Dbg::own("ModelClient"),
        }
    }
}
//
impl<'a> Eval<String, Result<String, Error>> for ModelClient<'a> {
    fn eval(&self, query: String) -> Result<String, Error> {
        let error = Error::new(&self.dbg, "eval");
        match self.ctx.eval(query) {
            Ok(ctx) => {
                let model_query = ctx; 
                let t = Instant::now();
                let socket = TcpStream::connect(&self.model_addr)
                    .map_err(|err| error.pass_with(format!("Can't connect to the Model '{}'", self.model_addr), err.to_string()))?;
                // - Make query to the Model using socket
                // - Wait for reply from the Model
                // - Return the reply string
                let mut reply = todo!();
                let elapsed = t.elapsed();
                log::debug!("{}.eval | Elapsed {:?}", self.dbg, elapsed);
                log::debug!("{}.eval | Model answer: {}", self.dbg, reply);
                Ok(reply)
            }
            Err(err) => Err(error.pass(err.to_string()))
        }
    }
}