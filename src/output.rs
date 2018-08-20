// Conserve backup system.
// Copyright 2018 Martin Pool.

//! Text output formats for structured data.
//!
//! These are objects that accept iterators of different types of content, and write it to a
//! file (typically stdout).

use super::*;

use chrono::Local;

/// Show something about an archive.
pub trait ShowArchive {
    fn show_archive(&self, &Archive) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct ShortVersionList {
}


impl ShowArchive for ShortVersionList {
    fn show_archive(&self, archive: &Archive) -> Result<()> {
        for band_id in archive.list_bands()? {
            println!("{}", band_id);
        }
        Ok(())
    }
}
 
#[derive(Debug, Default)]
pub struct VerboseVersionList {}

impl ShowArchive for VerboseVersionList {
    fn show_archive(&self, archive: &Archive) -> Result<()> {
        for band_id in archive.list_bands()? {
        let band = match Band::open(&archive, &band_id) {
            Ok(band) => band,
            Err(e) => {
                warn!("Failed to open band {:?}: {:?}", band_id, e);
                continue;
            }
        };
        let info = match band.get_info(archive.report()) {
            Ok(info) => info,
            Err(e) => {
                warn!("Failed to read band tail {:?}: {:?}", band_id, e);
                continue;
            }
        };
        let is_complete_str = if info.is_closed {
            "complete"
        } else {
            "incomplete"
        };
        let start_time_str = info.start_time.with_timezone(&Local).to_rfc3339();
        let duration_str = info.end_time.map_or_else(String::new, |t| {
            format!("{}s", (t - info.start_time).num_seconds())
        });
        println!(
            "{:<26} {:<10} {} {:>7}",
            band_id,
            is_complete_str,
            start_time_str,
            duration_str
        );
    }
    Ok(())
    }
}