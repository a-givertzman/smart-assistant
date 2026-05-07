use std::path::PathBuf;
use sal_core::error::Error;
use crate::{
    domain::Eval, 
    embedding::select_file_type::FileTypeSelector
};
//
/// Класс для извлечения текстового содержимого из документов различных форматов.
/// 
/// Объединяет логику выбора типа файла и запуска соответствующей стратегии извлечения.
pub struct TextExtraction {
    /// Путь к файлу, из которого необходимо извлечь текст.
    file_path: PathBuf,
    /// Путь к системной утилите `pdf2htmlEX` для обработки PDF-документов.
    pdf_tool: PathBuf,
}
//
impl TextExtraction {
    /// Создает новый экземпляр [TextExtraction].
    /// 
    /// # Аргументы
    /// * `file_path` - Путь к исходному документу (PDF, DOCX, HTML, TXT).
    /// * `pdf_tool` - Путь к исполняемому файлу/AppImage конвертера PDF.
    pub fn new(
        file_path: PathBuf,
        pdf_tool: PathBuf,
    ) -> Self {
        Self {
            file_path,
            pdf_tool,
        }
    }
}
//
impl Eval<(), Result<String, Error>> for TextExtraction {
    /// Выполняет полный цикл обработки документа: от определения типа до получения HTML-строки.
    /// 
    /// # Errors
    /// Возвращает [Error], если:
    /// - Формат файла не поддерживается.
    /// - Возникла ошибка при вызове внешней утилиты (pdf2htmlEX или Pandoc).
    /// - Не удалось прочитать содержимое файла с диска.
    fn eval(&self, _: ()) -> Result<String, Error> {
        // Выбор стратегии через селектор
        match FileTypeSelector::new(self.file_path.clone(), self.pdf_tool.clone()).eval(()) {
            Ok(converter) => {
                // Выполнение конвертации выбранным специалистом
                converter.eval(())
            },
            Err(e) => Err(e),
        }
    }
}
