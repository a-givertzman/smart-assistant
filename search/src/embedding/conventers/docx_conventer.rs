use std::path::PathBuf;
use sal_core::error::Error;
use crate::domain::Eval;
/// Конвертер документов формата DOCX в HTML.
/// 
/// Является специализированной реализацией стратегии извлечения текста 
/// для файлов Microsoft Word. Использует системную утилиту `pandoc`.
pub struct DocxConverter {
    /// Путь к исходному DOCX-файлу.
    file_path: PathBuf,
}
//
impl DocxConverter {
    /// Создает новый экземпляр [DocxConverter].
    /// 
    /// # Аргументы
    /// * `file_path` - Путь к DOCX-файлу, который необходимо обработать.
    pub fn new(file_path: PathBuf) -> Self {
        Self { 
            file_path 
        }
    }
}
//
impl Eval<(), Result<String, Error>> for DocxConverter {
    /// Выполняет конвертацию DOCX в HTML строку через Pandoc.
    /// 
    /// # Требования
    /// Для корректной работы метода в системе (или WSL окружении) должен быть 
    /// установлен пакет `pandoc` (`sudo apt install pandoc`).
    /// 
    /// # Errors
    /// Возвращает [Error], если:
    /// - Утилита `pandoc` не найдена в системном PATH.
    /// - Входной файл поврежден или недоступен для чтения.
    /// - Произошла ошибка во время выполнения процесса конвертации.
    fn eval(&self, _: ()) -> Result<String, Error> {
        let mut p = pandoc::new();
        p.add_input(&self.file_path);
        p.set_output_format(pandoc::OutputFormat::Html, vec![]);
        p.set_output(pandoc::OutputKind::Pipe);
        p.add_option(pandoc::PandocOption::SelfContained);
        p.add_option(pandoc::PandocOption::Standalone); 
        match p.execute() {
            Ok(pandoc::PandocOutput::ToBuffer(html)) => Ok(html),
            Ok(_) => Err(Error::from("Pandoc returned unexpected output format")),
            Err(e) => Err(Error::from(format!("Pandoc DOCX conversion failed: {:?}", e))) 
        }
    }
}
