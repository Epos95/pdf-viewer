// So the json structure is prolly just a HashMap<String, u16>
// storing the book name and the page its storing right now.

// the persistence could have a function running constantly checking the content dir for new pdfs
// so that new pdfs can be appended at runtime

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::{error::Error, ffi::OsStr};
use tokio::fs::read_dir;

use crate::{
    routes::stats::{ReadingStatistics, WrappedReadingStatistics},
    state::{Pdf, PdfCollection, WrappedPdfCollection},
};

// TODO: Maybe implement Drop for this so we dont get halfwrites when exiting the program
#[derive(Serialize, Deserialize)]
/// Struct for writing BOTH a pdfcollection and a readingstatistics to disc as one.
pub struct DiscState {
    pub pdfs: PdfCollection,
    pub reading_history: ReadingStatistics,
}

/// Syncs the state in memory with the state on disk.
/// Should run in the background continously.
pub async fn sync_state(
    content_dirs: Vec<PathBuf>,
    state_location: PathBuf,
    pdfs: WrappedPdfCollection,
    reading_history: WrappedReadingStatistics,
) -> Result<(), Box<dyn Error>> {
    // check `content_dir` for pdfs not in `state` and add them
    let mut state_ref = pdfs.lock().await;

    for content_dir in content_dirs {
        let mut files = read_dir(content_dir).await?;

        // TODO: Remove things from the state which are NOT within the directory.
        while let Ok(Some(f)) = files.next_entry().await {
            if f.path().extension() != Some(OsStr::new("pdf")) {
                continue;
            }

            let name = f.file_name().into_string().unwrap();
            let path = f.path();

            if !state_ref.has_book(&name.split('.').next().unwrap()) {
                tracing::info!("Added new book {path:?}");
                let doc = Pdf::new(path);
                state_ref.add_book(doc);
            }
        }
    }

    drop(state_ref);

    // write state to file
    // TODO: investigate if this not being async gives issues
    //       also how to make it async
    let fd = OpenOptions::new()
        .write(true)
        .read(false)
        .truncate(true)
        // TODO: enforce data location through a exported const
        //       also need to enforce the location of a specific
        //       directory to store the state in, like `~/.config``
        .open(state_location)?;

    let reading_history = &*reading_history.lock().await;
    let pdfs = &*pdfs.lock().await;
    let state = DiscState {
        pdfs: pdfs.clone(),
        reading_history: reading_history.clone(),
    };

    serde_json::to_writer_pretty(fd, &state)?;
    Ok(())
}
