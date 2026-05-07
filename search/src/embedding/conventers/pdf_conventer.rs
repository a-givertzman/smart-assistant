use std::{fs, path::PathBuf, process::Command};
use sal_core::error::Error;
use crate::domain::Eval;
/// Конвертер PDF-документов в HTML-формат.
/// 
/// Является специализированной реализацией стратегии извлечения текста 
/// для файлов формата PDF. Использует внешнюю утилиту `pdf2htmlEX`.
pub struct PdfConverter {
    /// Полный путь к исходному PDF-файлу.
    file_path: PathBuf,
    /// Путь к исполняемому файлу конвертера (обычно AppImage).
    converter_path: PathBuf,
}
//
impl PdfConverter {
    /// Создает новый экземпляр [PdfConverter].
    /// 
    /// # Аргументы
    /// * `file_path` - Путь к PDF, который нужно обработать.
    /// * `converter_path` - Путь к утилите `pdf2htmlEX`.
    pub fn new(file_path: PathBuf, converter_path: PathBuf) -> Self {
        Self {
            file_path,
            converter_path,
        }
    }
}
//
impl Eval<(), Result<String, Error>> for PdfConverter {
    /// Выполняет конвертацию PDF в HTML строку.
    /// 
    /// # Errors
    /// Возвращает [Error] в случаях:
    /// - Если не удалось запустить процесс конвертации (неверный путь к утилите).
    /// - Если `pdf2htmlEX` завершился с ненулевым кодом возврата (ошибка парсинга PDF).
    /// - Если не удалось прочитать результат или очистить временные данные.
    fn eval(&self, _: ()) -> Result<String, Error> {
        let temp_dir = std::path::Path::new("/tmp");
        let thread_id = std::thread::current().id();
        let temp_html_name = format!("temp_output_{:?}.html", thread_id);
        let temp_html_path = temp_dir.join(&temp_html_name);
        // Запуск внешнего процесса
        let output = Command::new(self.converter_path.clone())
            .arg("--appimage-extract-and-run")
            .arg("--dest-dir").arg(temp_dir)
            .arg(self.file_path.clone())       
            .arg(&temp_html_name)              
            .output()
            .map_err(|e| Error::from(format!("Exec error: {:?}", e)))?;
        // Валидация завершения процесса
        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            let out_msg = String::from_utf8_lossy(&output.stdout);
            return Err(Error::from(format!("pdf2htmlEX error: {} {}", err_msg, out_msg)));
        }
        // Чтение данных и пост-обработка
        let html_content = fs::read_to_string(&temp_html_path)
            .map_err(|e| Error::from(format!("Read error from /tmp: {:?}", e)))?;
        // Гарантированная очистка временных ресурсов
        let _ = fs::remove_file(&temp_html_path);
        Ok(html_content)
    }
}
