#[cfg(test)]
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

use crate::{domain::Eval, embedding::text_extraction::text_extraction_eval::TextExtractionEval};
///
///
static INIT: Once = Once::new();
///
/// once called initialisation
fn init_once() {
    INIT.call_once(|| {
        // implement your initialisation code to be called only once for current test file
    })
}
///
/// returns:
///  - ...
fn init_each() -> () {}
///
/// Testing such functionality / behavior
#[test]
fn subject() {
    DebugSession::new()
        .filter(LogLevel::Debug)
        .module("module-name::sub::path::Class", LogLevel::Info)
        .init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("search-test-subject");
    log::debug!("\n{dbg}");
    let test_duration = TestDuration::new(dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (
            1,
            ""
        )
    ];
    for (step, text_path) in test_data {
        let result = TextExtractionEval::new().eval(text_path.to_owned());
        // assert!(result == target, "{dbg} | step {step} \nresult: {:?}\ntarget: {:?}", result, target);
    }
    test_duration.exit();
}
