use std::cmp::Ordering;

use chrono::NaiveDateTime;
use pdf_viewer::state::Pdf;
use ratatui::widgets::TableState;

impl StatefulTable {
    pub fn with_items(items: Vec<TableItem>) -> Self {
        let mut state = TableState::default();

        if items.len() > 0 {
            state.select(Some(0));
        }

        Self {
            state,
            items,
            header_index: 0,
            sort_direction: SortDirection::None,
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn next_header(&mut self) {
        let len = self.items.get(0).map_or(4, |v| v.as_vec().len()) - 1;
        if self.header_index < len {
            self.header_index += 1;
        }
    }

    pub fn previous_header(&mut self) {
        if self.header_index > 0 {
            self.header_index -= 1;
        }
    }

    pub fn header_index(&self) -> usize {
        self.header_index
    }

    pub fn sort_direction(&self) -> SortDirection {
        self.sort_direction
    }

    pub fn next_sort_direction(&mut self) {
        let new = match self.sort_direction {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::None,
            SortDirection::None => SortDirection::Ascending,
        };

        self.sort_direction = new;
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum SortDirection {
    None,
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn to_string(&self) -> String {
        match self {
            Self::Ascending => String::from(" ▼"),
            Self::Descending => String::from(" ▲"),
            Self::None => String::new(),
        }
    }

    pub fn get_comparison(
        &self,
        index: usize,
    ) -> Option<impl FnMut(&TableItem, &TableItem) -> Ordering> {
        match self {
            SortDirection::None => None,
            SortDirection::Ascending => Some(match index {
                0 => |a: &TableItem, b: &TableItem| -> Ordering { a.pdf_name().cmp(b.pdf_name()) },
                1 => |a: &TableItem, b: &TableItem| -> Ordering { a.cur_page().cmp(&b.cur_page()) },
                2 => |a: &TableItem, b: &TableItem| -> Ordering {
                    a.total_pages().cmp(&b.total_pages())
                },
                3 => |a: &TableItem, b: &TableItem| -> Ordering {
                    let first = NaiveDateTime::parse_from_str(&a.last_access, "%Y-%m-%d %H:%M:%S")
                        .unwrap_or_default();
                    let second = NaiveDateTime::parse_from_str(&b.last_access, "%Y-%m-%d %H:%M:%S")
                        .unwrap_or_default();
                    first.cmp(&second)
                },
                _ => unreachable!(),
            }),
            SortDirection::Descending => Some(match index {
                0 => |a: &TableItem, b: &TableItem| -> Ordering { b.pdf_name().cmp(a.pdf_name()) },
                1 => |a: &TableItem, b: &TableItem| -> Ordering { b.cur_page().cmp(&a.cur_page()) },
                2 => |a: &TableItem, b: &TableItem| -> Ordering {
                    b.total_pages().cmp(&a.total_pages())
                },
                3 => |a: &TableItem, b: &TableItem| -> Ordering {
                    let first = NaiveDateTime::parse_from_str(&a.last_access, "%Y-%m-%d %H:%M:%S")
                        .unwrap_or_default();
                    let second = NaiveDateTime::parse_from_str(&b.last_access, "%Y-%m-%d %H:%M:%S")
                        .unwrap_or_default();
                    second.cmp(&first)
                },
                _ => unreachable!(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatefulTable {
    pub(crate) state: TableState,
    pub(crate) items: Vec<TableItem>,

    header_index: usize,
    sort_direction: SortDirection,
}

#[derive(Debug, Clone)]
pub struct TableItem {
    pdf_name: String,
    cur_page: u16,
    total_pages: u16,
    last_access: String,
}

impl TableItem {
    pub fn as_vec(&self) -> Vec<String> {
        vec![
            self.pdf_name.clone(),
            self.cur_page.to_string(),
            self.total_pages.to_string(),
            self.last_access.clone(),
        ]
    }

    pub fn pdf_name(&self) -> &str {
        self.pdf_name.as_ref()
    }

    pub fn cur_page(&self) -> u16 {
        self.cur_page
    }

    pub fn total_pages(&self) -> u16 {
        self.total_pages
    }

    pub fn last_access(&self) -> &str {
        self.last_access.as_ref()
    }
}

impl From<Pdf> for TableItem {
    fn from(p: Pdf) -> Self {
        Self {
            pdf_name: p.name().to_string(),
            cur_page: p.current_page(),
            total_pages: p.total_pages(),
            last_access: p.last_access().to_string(),
        }
    }
}
