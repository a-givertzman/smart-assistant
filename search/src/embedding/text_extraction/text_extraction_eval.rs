use std::path::{
    Path, 
    PathBuf
};
use pdf_oxide::{
    converters::{
        ConversionOptions, 
        ReadingOrderMode
    }, 
    pipeline::BoldMarkerBehavior
};
use sal_core::error::Error;
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
    fn extract_pdf(&self, path: &Path) -> Result<String, Error> {
        match pdf_oxide::PdfDocument::open(path) {
            Ok(doc) => {
                match doc.to_html_all(
                    &ConversionOptions {
                        preserve_layout: false,
                        detect_headings: true,
                        extract_tables: true,
                        include_images: true,
                        image_output_dir: Some("D:\\work_projects\\smart-assistant\\search\\src\\tests\\unit\\test_files".to_owned()),
                        embed_images: true,
                        reading_order_mode: ReadingOrderMode::StructureTreeFirst { mcid_order: vec![] },
                        bold_marker_behavior: BoldMarkerBehavior::Conservative,
                        table_detection_config: None,
                        render_formulas: false,
                        page_images: None,
                        page_dimensions: None,
                        include_form_fields: true,
                        max_image_pixels: None,
                    }
                ) {
                    Ok(html_result) => {
                        Ok(html_result)
                    },
                    Err(e) => Err(Error::from(format!("Error to converse pdf file to html: {:?}", e))) 
                }
            },
            Err(e) => Err(Error::from(format!("Error to open pdf file: {:?}", e))) 
        }
    }
    fn extract_docx(&self, path: &Path) -> Result<String, Error> {
        match rdocx::Document::open(path) {
            Ok(doc) => {
                return  Ok(doc.to_html())
            },
            Err(e) => Err(Error::from(format!("Error to open docx file: {:?}", e)))
        }
    }
    fn extract_html(&self, path: &Path) -> Result<String, Error> {
        match std::fs::read_to_string(path) {
            Ok(html) => {
                    let html_text_result = scraper::Html::parse_document(&html);
                    return Ok(html_text_result.html());
            },
            Err(e) => Err(Error::from(format!("Error to read html file: {:?}", e)))
        }
    }
}
//
//
impl Eval<PathBuf,  Result<String, Error>> for TextExtractionEval {
    fn eval(&self, file_path: PathBuf) ->  Result<String, Error> {
        match file_path.extension().and_then(|s| s.to_str()) {
            Some("pdf") => self.extract_pdf(&file_path),
            Some("docx") => self.extract_docx(&file_path),
            Some("html") | Some("htm") => self.extract_html(&file_path),
            Some("txt") => Ok(std::fs::read_to_string(file_path).unwrap_or_default()),
            _ => panic!("Unsupported file format"),
        }
    }
}