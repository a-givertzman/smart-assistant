use std::{io::{Read, Write}, net::TcpListener, sync::Arc, time::{Duration, Instant}};

use sal_core::{dbg::Dbg, error::Error};

use crate::domain::Eval;

///
/// Starts a TCP Server
/// - listening for Clint's requests
/// - Returns Request as text for now
pub struct Server<'a> {
    addr: String,
    ctx: Arc<Box<dyn Eval<String, Result<String, Error>> + Send + Sync + 'a>>,
    dbg: Dbg,
}
//
impl<'a> Server<'a> {
    ///
    /// Returns [Server] new instance
    pub fn new(
        addr: impl Into<String>,
        ctx: impl Eval<String, Result<String, Error>> + Send + Sync + 'a,
    ) -> Self {
        Self {
            addr: addr.into(),
            ctx: Arc::new(Box::new(ctx)),
            dbg: Dbg::own("Server"),
        }
    }
}
//
impl<'a> Eval<(), Result<(), Error>> for Server<'a> {
    fn eval(&self, _: ()) -> Result<(), Error> {
        loop {
            match TcpListener::bind(&self.addr) {
                Ok(listener) => {
                    log::debug!("{}.eval | Server ready on {:?}", self.dbg, self.addr);
                    match listener.accept() {
                        Ok((mut socket, addr)) => {
                            log::debug!("{}.eval | Clint {:?} connected", self.dbg, addr);
                            'read: loop {
                                let t = Instant::now();
                                let mut query = String::new();
                                match socket.read_to_string(&mut query) {
                                    Ok(len) => {
                                        log::debug!("{}.eval | Incoming query {:?}", self.dbg, query);
                                        match self.ctx.eval(query) {
                                            Ok(ctx) => {
                                                let reply = ctx;
                                                match socket.write_all(reply.as_bytes()) {
                                                    Ok(_) => {},
                                                    Err(err) => {
                                                        log::warn!("{}.eval | Can't send reply to {addr}: {:?}", self.dbg, err);
                                                        break 'read;
                                                    }
                                                }
                                                let elapsed = t.elapsed();
                                                log::debug!("{}.eval | Elapsed {:?}", self.dbg, elapsed);
                                                log::debug!("{}.eval | Model answer: {}", self.dbg, reply);
                                            }
                                            Err(err) => {
                                                log::warn!("{}.eval | Can't perform request from {addr}: {:?}", self.dbg, err);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        log::warn!("{}.eval | Can't read request from {addr}: {:?}", self.dbg, err);
                                        break 'read;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            log::warn!("{}.eval | Can't accept request, \n\terror: {:?}", self.dbg, err);
                        }
                    }
                },
                Err(err) => {
                    log::warn!("{}.eval | Can't setup TCP Server on '{}', \n\terror: {:?}", self.dbg, self.addr, err);
                    std::thread::sleep(Duration::from_secs(3));
                },
            };
        }
    }
}