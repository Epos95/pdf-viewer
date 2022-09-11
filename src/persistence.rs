// So the json structure is prolly just a HashMap<String, u16>
// storing the book name and the page its storing right now.

// the persistence could have a function running constantly checking the content dir for new pdfs
// so that new pdfs can be appended at runtime

use std::error::Error;
use std::fs::OpenOptions;
use std::path::PathBuf;
use tokio::fs::read_dir;
use tracing::info;

use crate::ContentState;

/// Syncs the state in memory with the state on disk.
/// Should run in the background continously.
pub async fn sync_state(content_dir: PathBuf, state_location: PathBuf, state: ContentState) -> Result<(), Box<dyn Error>> {
    // check `content_dir` for pdfs not in `state` and add them
    let mut guard = state.lock().await;
    let mut files = read_dir(content_dir).await?;

    // TODO: Remove things from the state which are NOT within the directory.
    while let Ok(Some(f)) = files.next_entry().await {
        let string = f.file_name().into_string().unwrap();
        if !guard.contains_key(&string) {
            // does not contain the string

            info!("Found new file: {string}");
            guard.insert(string, 1);
        }
    }
    drop(guard);

    // write state to file
    // TODO: investigate if this not being async gives issues
    // 		 also how to make it async
    let fd = OpenOptions::new()
        .write(true)
        .read(false)
        .truncate(true)
        // TODO: enforce data location through a exported const
        //       also need to enforce the location of a specific 
        //       directory to store the state in, like `~/.config``
        .open(state_location)?;

    let hmap = &*state.lock().await;

    serde_json::to_writer(fd, &hmap)?;
    Ok(())
}
