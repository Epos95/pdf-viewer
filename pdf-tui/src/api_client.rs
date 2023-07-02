use pdf_viewer::routes::main_page::MainTemplate;
use pdf_viewer::state::Pdf;

use crate::app::stateful_table::TableItem;

#[derive(Debug)]
pub enum ApiError {
    NoConnection(String),
    InvalidResponse,
}

impl Into<String> for ApiError {
    fn into(self) -> String {
        match self {
            ApiError::InvalidResponse => String::from("Failed to parse JSON response"),
            ApiError::NoConnection(s) => format!("Failed to connect to server at: \"{s}\""),
        }
    }
}

pub struct ApiClient {
    connection_ip: String,
    pdf_list: Vec<Pdf>,
    read_history: (usize, usize, usize),
}

impl ApiClient {
    pub fn new(ip: String) -> Self {
        Self {
            connection_ip: ip,
            pdf_list: vec![],
            read_history: (0, 0, 0),
        }
    }

    pub async fn refresh(&mut self) -> Result<(), ApiError> {
        let ip = format!("{}/api/", self.connection_ip);

        let template = reqwest::get(&ip)
            .await
            .map_err(|_| ApiError::NoConnection(ip))?
            .json::<MainTemplate>()
            .await
            .map_err(|_| ApiError::InvalidResponse)?;

        self.pdf_list = template.pdfs().to_vec();
        self.read_history = (template.month(), template.week(), template.today());

        Ok(())
    }

    pub fn pdfs_as_table_item(&self) -> Vec<TableItem> {
        self.pdf_list.iter().map(|p| (*p).clone().into()).collect()
    }

    pub fn connection_ip(&self) -> &str {
        self.connection_ip.as_ref()
    }

    pub fn read_history(&self) -> (usize, usize, usize) {
        self.read_history
    }
}
