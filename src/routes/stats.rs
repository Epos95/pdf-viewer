// Route for counting how much has been read in the last `n` days

use std::sync::Arc;
use tokio::sync::Mutex;

use axum::{response::IntoResponse, Extension};
use chrono::{DateTime, Duration, Local};

#[derive(PartialEq, Clone, Copy)]
#[allow(dead_code)]
enum ReadCategory {
    Day,
    Week,
    Month,
    More,
}

struct ReadingEvent {
    time: DateTime<Local>,
    validity: ReadCategory,
}

impl Default for ReadingEvent {
    fn default() -> Self {
        Self {
            time: Local::now(),
            validity: ReadCategory::Day,
        }
    }
}

impl ReadingEvent {
    /// Updates a `ReadingEvent`s `ReadCategory` and return the new `ReadCategory`.
    pub fn update(&mut self, current_time: &DateTime<Local>) {
        let time_since = self.time - *current_time;

        if time_since < Duration::days(1) {
            self.validity = ReadCategory::Day;
        } else if time_since < Duration::weeks(1) {
            self.validity = ReadCategory::Week;
        } else if time_since < Duration::weeks(4) {
            self.validity = ReadCategory::Month;
        } else {
            self.validity = ReadCategory::More;
        }
    }
}

pub type WrappedReadingStatistics = Arc<Mutex<ReadingStatistics>>;
pub struct ReadingStatistics {
    events: Vec<ReadingEvent>,
}

impl ReadingStatistics {
    pub fn new() -> Self {
        Self { events: vec![] }
    }

    pub fn wrapped() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new()))
    }

    pub fn increment(&mut self) {
        self.events.push(ReadingEvent::default())
    }

    pub fn update(&mut self) {
        let now = Local::now();

        // Update all events
        self.events.iter_mut().for_each(|e| e.update(&now));

        // Removes the events which are no longer valid
        self.events.retain(|e| e.validity != ReadCategory::More);
    }

    pub fn last_day(&self) -> usize {
        self.events
            .iter()
            .filter(|e| e.validity == ReadCategory::Day)
            .count()
    }

    pub fn last_week(&self) -> usize {
        self.events
            .iter()
            .filter(|e| e.validity == ReadCategory::Week)
            .count()
    }

    pub fn last_month(&self) -> usize {
        self.events
            .iter()
            .filter(|e| e.validity == ReadCategory::Month)
            .count()
    }
}

pub async fn get_last_day(
    Extension(reading_stats): Extension<WrappedReadingStatistics>,
) -> impl IntoResponse {
    "200"
}
pub async fn get_last_week(
    Extension(reading_stats): Extension<WrappedReadingStatistics>,
) -> impl IntoResponse {
    "200"
}
pub async fn get_last_month(
    Extension(reading_stats): Extension<WrappedReadingStatistics>,
) -> impl IntoResponse {
    "200"
}
