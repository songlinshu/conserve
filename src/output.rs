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
    fn show_archive(&self, _: &Archive) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct ShortVersionList {}

impl ShowArchive for ShortVersionList {
    fn show_archive(&self, archive: &Archive) -> Result<()> {
        for band_id in archive.list_bands()? {
            println!("{}", band_id);
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct VerboseVersionList {
    show_sizes: bool,
}

impl VerboseVersionList {
    // Control whether to show the size of version disk usage.
    //
    // Setting this requires walking the band directories which takes some extra time.
    pub fn show_sizes(self, show_sizes: bool) -> VerboseVersionList {
        VerboseVersionList { show_sizes }
    }
}

impl ShowArchive for VerboseVersionList {
    fn show_archive(&self, archive: &Archive) -> Result<()> {
        let report = archive.report();
        for band_id in archive.list_bands()? {
            let band = match Band::open(&archive, &band_id) {
                Ok(band) => band,
                Err(e) => {
                    report.problem(&format!("Failed to open band {:?}: {:?}", band_id, e));
                    continue;
                }
            };
            let info = match band.get_info(archive.report()) {
                Ok(info) => info,
                Err(e) => {
                    report.problem(&format!("Failed to read band tail {:?}: {:?}", band_id, e));
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
            if self.show_sizes {
                let disk_bytes = band.get_disk_size()?;
                println!(
                    "{:<26} {:<10} {} {:>7} {:>8}MB",
                    band_id,
                    is_complete_str,
                    start_time_str,
                    duration_str,
                    disk_bytes / 1_000_000,
                );
            } else {
                println!(
                    "{:<26} {:<10} {} {:>7}",
                    band_id, is_complete_str, start_time_str, duration_str,
                );
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct IndexDump {
    band_id: String,
}

impl IndexDump {
    pub fn new(band_id: &str) -> Self {
        Self {
            band_id: band_id.to_string(),
        }
    }
}

impl ShowArchive for IndexDump {
    fn show_archive(&self, archive: &Archive) -> Result<()> {
        let report = archive.report();
        let band_id = BandId::from_string(&self.band_id)?;
        let band = match Band::open(&archive, &band_id) {
            Ok(band) => band,
            Err(e) => {
                report.problem(&format!("Failed to open band {:?}: {:?}", band_id, e));
                return Err(e);
            }
        };
        let index_entries = band
            .index()
            .iter(&excludes::excludes_nothing(), &report)
            .unwrap()
            .filter_map(|i| i.ok())
            .collect::<Vec<Entry>>();
        let output = serde_json::to_string_pretty(&index_entries)?;
        report.print(&output);
        Ok(())
    }
}
