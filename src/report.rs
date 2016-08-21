// Conserve backup system.
// Copyright 2015, 2016 Martin Pool.

//! Count interesting events that occur during a run.

use std::collections;

#[allow(unused)]
static KNOWN_COUNTERS: &'static [&'static str] = &[
    "backup.file.count",
    "backup.skipped.unsupported_file_kind",
    "block.read.count",
    "block.read.corrupt",
    "block.read.misplaced",
    "block.write.already_present",
    "block.write.compressed_bytes",
    "block.write.count",
    "block.write.uncompressed_bytes",
    "index.write.compressed_bytes",
    "index.write.uncompressed_bytes",
    "index.write.hunks",
    "source.selected.count",
    "source.skipped.unsupported_file_kind",
    "source.visited.directories.count",
];

/// A Report is notified of problems or non-problematic events that occur while Conserve is
/// running.
///
/// A Report holds counters, identified by a name.  The name must be in `KNOWN_COUNTERS`.
#[derive(Clone, Debug)]
pub struct Report {
    count: collections::HashMap<&'static str, u64>,
}

impl Report {
    pub fn new() -> Report {
        let mut count = collections::HashMap::with_capacity(KNOWN_COUNTERS.len());
        for counter_name in KNOWN_COUNTERS {
            count.insert(*counter_name, 0);
        }
        Report { count: count }
    }

    /// Increment a counter by a given amount.
    ///
    /// The name must be a static string.  Counters implicitly start at 0.
    pub fn increment(self: &mut Report, counter_name: &'static str, delta: u64) {
        // Entries are created from the list of known names when this is constructed.
        if let Some(mut c) = self.count.get_mut(counter_name) {
            *c += delta;
        } else {
            panic!("unregistered counter {:?}", counter_name);
        }
    }

    /// Return the value of a counter.  A counter that has not yet been updated is 0.
    #[allow(unused)]
    pub fn get_count(self: &Report, counter_name: &str) -> u64 {
        *self.count.get(counter_name).unwrap_or(&0)
    }

    /// Merge the contents of `from_report` into `self`.
    pub fn merge_from(self: &mut Report, from_report: &Report) {
        for (name, value) in &from_report.count {
            self.increment(name, *value);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::Report;

    #[test]
    pub fn count() {
        let mut r = Report::new();
        assert_eq!(r.get_count("block.read.count"), 0);
        r.increment("block.read.count", 1);
        assert_eq!(r.get_count("block.read.count"), 1);
        r.increment("block.read.count", 10);
        assert_eq!(r.get_count("block.read.count"), 11);
    }

    #[test]
    pub fn merge_reports() {
        let mut r1 = Report::new();
        let mut r2 = Report::new();
        r1.increment("block.read.count", 1);
        r1.increment("block.read.corrupt", 2);
        r2.increment("block.write.count", 1);
        r2.increment("block.read.corrupt", 10);
        r1.merge_from(&r2);
        assert_eq!(r1.get_count("block.read.count"), 1);
        assert_eq!(r1.get_count("block.read.corrupt"), 12);
        assert_eq!(r1.get_count("block.write.count"), 1);
    }
}
