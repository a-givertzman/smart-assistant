use std::path::PathBuf;
use sal_core::error::Error;
use crate::{
    domain::Eval, 
    embedding::conventers::{
        docx_conventer::DocxConverter, html_conventer::HtmlConverter, pdf_conventer::PdfConverter
    }
};
/// Определения типа (расширения) документа и выбора соответствующего конвертера.
pub struct FileTypeSelector {
    /// Путь к файлу, тип которого необходимо определить.
    file_path: PathBuf, 
    /// Путь к системной утилите для обработки PDF (pdf2htmlEX).
    pdf_tool: PathBuf,
}
//
impl FileTypeSelector {
    /// Создает новый экземпляр [FileTypeSelector].
    /// 
    /// # Аргументы
    /// * `file_path` - Путь к исходному документу.
    /// * `pdf_tool` - Путь к AppImage или исполняемому файлу для конвертации PDF.
    pub fn new(file_path: PathBuf, pdf_tool: PathBuf) -> Self {
        Self { 
            file_path, 
            pdf_tool
        }
    }
}
//
impl Eval<(), Result<Box<dyn Eval<(), Result<String, Error>>>, Error>> for FileTypeSelector {
    /// Анализирует расширение файла и возвращает конкретную реализацию конвертера.
    /// 
    /// ### Логика выбора:
    /// - `.pdf`  => [PdfConverter]
    /// - `.docx` => [DocxConverter]
    /// - `.html` / `.htm` => [HtmlConverter]
    /// 
    /// # Errors
    /// Возвращает [Error], если расширение файла отсутствует или не входит 
    /// в список поддерживаемых форматов.
    fn eval(&self, _: ()) -> Result<Box<dyn Eval<(), Result<String, Error>>>, Error> {
        let ext = self.file_path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        match ext.as_deref() {
            Some("pdf") => Ok(
                Box::new(
                    PdfConverter::new(
                        self.file_path.clone(), 
                        self.pdf_tool.clone()
                    )
                )
            ),
            Some("docx") => Ok(
                Box::new(
                    DocxConverter::new(self.file_path.clone()) 
                )
            ),
            Some("html") | Some("htm") => Ok(
                Box::new(
                    HtmlConverter::new(self.file_path.clone())
                )
            ),
            _ => Err(Error::from(format!("Unsupported format: {:?}", ext))),
        }
    }
}
