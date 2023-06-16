use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AccessTime {
    Never,
    Once(String),
}

impl AccessTime {
    pub fn now() -> Self {
        AccessTime::Once(Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
    }
}

impl Display for AccessTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessTime::Never => {
                write!(f, "never")
            }
            AccessTime::Once(s) => {
                write!(f, "{s}")
            }
        }
    }
}

impl From<DateTime<Local>> for AccessTime {
    fn from(value: DateTime<Local>) -> Self {
        AccessTime::Once(value.to_string())
    }
}

pub type WrappedPdfCollection = Arc<Mutex<PdfCollection>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfCollection {
    // Use hashmap instead for that sweet, sweet, k,v goodness. Also because JSON prefers it.
    pub pdfs: HashMap<String, Pdf>,
}

impl PdfCollection {
    pub fn wrapped(self) -> WrappedPdfCollection {
        Arc::new(Mutex::new(self))
    }

    pub fn get_book_by_name_mut<S: Into<String> + Display + ?Sized>(
        &mut self,
        name: &S,
    ) -> Option<&mut Pdf> {
        let stringed = name.to_string();
        let name = stringed.strip_suffix(".pdf").unwrap_or(&stringed);
        self.pdfs.get_mut(name)
    }

    pub fn get_book_by_name<S: Into<String> + Display + ?Sized>(&self, name: &S) -> Option<Pdf> {
        let stringed = name.to_string();
        let name = stringed.strip_suffix(".pdf").unwrap_or(&stringed);
        self.pdfs.get(name).cloned()
    }

    pub fn set_page_by_name<S: Into<String> + Display + ?Sized>(
        &mut self,
        name: &S,
        new_page: u16,
    ) -> Option<()> {
        let stringed = name.to_string();
        let name = stringed.strip_suffix(".pdf").unwrap_or(&stringed);
        let pdf = self.pdfs.get_mut(name)?;

        pdf.current_page = new_page;

        Some(())
    }

    pub fn has_book<S: Into<String> + Display + ?Sized>(&self, name: &S) -> bool {
        self.get_book_by_name(name).is_some()
    }

    pub fn add_book(&mut self, pdf: Pdf) {
        self.pdfs.insert(pdf.name.clone(), pdf);
    }

    pub fn pdfs(&self) -> HashMap<String, Pdf> {
        self.pdfs.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pdf {
    last_access: AccessTime,
    name: String,
    path: PathBuf,
    current_page: u16,
    total_pages: u16,
}

impl Pdf {
    pub fn new(path: PathBuf) -> Pdf {
        let name = path
            .file_stem()
            .expect("Failed to extract the filename from {path:?}")
            .to_str()
            .expect("Couldnt convert {path} to string!")
            .to_string();

        let total_pages = Pdf::get_total_pages(path.as_path()).unwrap();
        tracing::info!("{name} has {total_pages} pages");

        Pdf {
            last_access: AccessTime::Never,
            name,
            path,
            current_page: 1,
            total_pages,
        }
    }

    /// Reads a pdf and gets the total pages in it.
    /// Fails on invalid files.
    fn get_total_pages(path: &Path) -> Result<u16, String> {
        // Memory inefficient but gets the job done...
        match lopdf::Document::load(path) {
            Ok(p) => Ok(p.get_pages().into_iter().len() as u16),
            Err(_) => Err(String::from("PDF not found at location: {path}")),
        }
    }

    pub fn last_access(&self) -> &AccessTime {
        &self.last_access
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn current_page(&self) -> u16 {
        self.current_page
    }

    pub fn total_pages(&self) -> u16 {
        self.total_pages
    }

    pub fn access(&mut self) {
        self.last_access = AccessTime::now();
    }

    pub fn percentage_read(&self) -> u32 {
        ((self.current_page as f32 / self.total_pages as f32) * 100.0).floor() as u32
    }
}
