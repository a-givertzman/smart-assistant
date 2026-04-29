use std::path::PathBuf;
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
            PathBuf::from("D:\\work_projects\\smart-assistant\\search\\src\\tests\\unit\\test_files\\pdf-test-1.pdf") // неправильная структура САМОГО ФАЙЛА
        ),
        (
            2,
            PathBuf::from("D:\\work_projects\\smart-assistant\\search\\src\\tests\\unit\\test_files\\pdf-test-2.pdf")
        ),
        (
            3,
            PathBuf::from("D:\\work_projects\\smart-assistant\\search\\src\\tests\\unit\\test_files\\docx-test-1.docx")
        ),
        (
            4,
            PathBuf::from("D:\\work_projects\\smart-assistant\\search\\src\\tests\\unit\\test_files\\docx-test-2.docx") // не парсится гистограмма
        )
    ];
    for (step, text_path) in test_data {
        match TextExtractionEval::new().eval(text_path.clone()) {
            Ok(result) => {
                let file_extenstion = text_path.extension().and_then(|s| s.to_str()).expect("Wrong file extension!");
                let result_file_path = format!("D:\\work_projects\\smart-assistant\\search\\src\\tests\\unit\\output_files\\result_{}_{:?}.html", file_extenstion, step);
                std::fs::File::create(&result_file_path).unwrap();
                std::fs::write(&result_file_path, result).unwrap();
                // assert!(result == target, "{dbg} | step {step} \nresult: {:?}\ntarget: {:?}", result, target);
            },
            Err(e) => log::error!("{}", format!("Step: {:?} Error to parse file: {:?}", step, e)),
        }
    }
    test_duration.exit();
}
