use std::{fs, path::PathBuf};
use sal_core::error::Error;
use crate::domain::Eval;
/// Конвертер и нормализатор HTML-документов.
/// 
/// Используется для обработки уже существующих HTML/HTM файлов. 
/// Обеспечивает корректное чтение исходного кода и его нормализацию 
/// с использованием парсера `scraper`.
pub struct HtmlConverter {
    /// Путь к исходному HTML-файлу.
    file_path: PathBuf,
}
//
impl HtmlConverter {
    /// Создает новый экземпляр [HtmlConverter].
    /// 
    /// # Аргументы
    /// * `file_path` - Путь к HTML-документу для обработки.
    pub fn new(file_path: PathBuf) -> Self {
        Self { 
            file_path 
        }
    }
}
//
impl Eval<(), Result<String, Error>> for HtmlConverter {
    /// Выполняет чтение и парсинг HTML-файла.
    /// 
    /// 
    /// # Errors
    /// Возвращает [Error], если:
    /// - Файл отсутствует по указанному пути.
    /// - Недостаточно прав для чтения файла.
    /// - Содержимое файла не является валидной UTF-8 строкой.
    fn eval(&self, _: ()) -> Result<String, Error> {
        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| Error::from(format!("Failed to read HTML: {:?}", e)))?;
        let document = scraper::Html::parse_document(&content);
        let selector = scraper::Selector::parse("p, h1, h2, h3, li")
            .map_err(|_| Error::from("Invalid selector"))?;
        let mut extracted_text = String::new();
        for element in document.select(&selector) {
            let text = element.text().collect::<Vec<_>>().join(" ");
            extracted_text.push_str(&text);
            extracted_text.push_str("\n\n");
        }
        Ok(extracted_text.trim().to_string())
    }
}
