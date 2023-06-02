use std::{
    fmt::Display,
    fs::File,
    io::Read,
    path::{Display as OtherDisplay, Path, PathBuf},
    sync::Arc,
};

use chrono::serde::ts_seconds;
use chrono::DateTime;
use chrono::Local;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AccessTime {
    Never,
    Once(String),
}

impl AccessTime {
    pub fn now() -> Self {
        AccessTime::Once(Local::now().to_string())
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
    pdfs: Vec<Pdf>,
}

impl PdfCollection {
    pub fn wrapped(self) -> WrappedPdfCollection {
        Arc::new(Mutex::new(self))
    }

    pub fn get_book_by_name<S: Into<String> + Display>(&self, name: S) -> Pdf {
        self.pdfs
            .iter()
            .find(|s| &s.name == &name.to_string())
            .expect("Uknown pdf name: {name}")
            .clone()
    }

    pub fn has_book<S: Into<String> + Display>(&self, name: S) -> bool {
        self.pdfs
            .iter()
            .find(|s| &s.name == &name.to_string())
            .is_some()
    }

    pub fn add_book(&mut self, pdf: Pdf) {
        self.pdfs.push(pdf);
    }

    pub fn pdfs(&self) -> &[Pdf] {
        self.pdfs.as_ref()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Pdf {
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

        let total_pages = Pdf::get_total_pages(path.as_path());
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
    fn get_total_pages(path: &Path) -> u16 {
        let re = Regex::new(r"/Type\s*/Page[^s]").unwrap();

        let mut file = File::open(path).unwrap();
        let mut buf = String::new();
        let n = file.read_to_string(&mut buf).unwrap();
        tracing::debug!("Read {n} bytes from {path:?}");

        re.find_iter(buf.as_str()).count().try_into().unwrap()
    }

    pub fn last_access(&self) -> &AccessTime {
        &self.last_access
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn current_page(&self) -> u16 {
        self.current_page
    }

    pub fn total_pages(&self) -> u16 {
        self.total_pages
    }
}
