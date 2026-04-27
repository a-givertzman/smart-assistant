use std::path::PathBuf;
use tesseract::Tesseract;
use crate::domain::Eval;
///
/// Text extraction
pub struct TextExtractionEval {

}
//
//
impl TextExtractionEval {
    ///
    /// New instance [TextExtractionEval]
    pub fn new() -> Self {
        Self { 

        }
    }
    fn extract_pdf_ocr(&self, path: &Path) -> String {
        Tesseract::new(None, Some("eng"))
            .and_then(|mut t| Ok(t.set_image(path)))
            .and_then(|mut t| t.get_text())
            .unwrap_or_default()
    }
}
//
//
impl Eval<String, String> for TextExtractionEval {
    fn eval(&self, val: String) -> String {
        let path = PathBuf::from(&val);
        match path.extension().and_then(|s| s.to_str()) {
            Some("pdf") => self.extract_pdf(path),
            Some("docx") => self.extract_docx(path),
            Some("html") | Some("htm") => self.extract_html(path),
            Some("txt") => std::fs::read_to_string(path).unwrap_or_default(),
            _ => panic!("Unsupported file format"),
        }
    }
}