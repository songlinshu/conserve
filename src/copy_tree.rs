// Conserve backup system.
// Copyright 2017, 2018, 2019, 2020 Martin Pool.

//! Copy tree contents.

use crate::kind::Kind;
use crate::stats::CopyStats;
use crate::*;

#[derive(Default, Clone, Debug)]
pub struct CopyOptions {
    pub print_filenames: bool,
    pub measure_first: bool,
    pub only_subtree: String,
}

/// Copy files and other entries from one tree to another.
///
/// NOTE: Although this is public, it's suggested to use `Archive::backup` or `Archive::restore` if
/// possible, as they're higher-level APIs.
pub fn copy_tree<ST: ReadTree, DT: WriteTree>(
    source: &ST,
    mut dest: DT,
    options: &CopyOptions,
) -> Result<CopyStats> {
    let mut stats = CopyStats::default();
    // This causes us to walk the source tree twice, which is probably an acceptable option
    // since it's nice to see realistic overall progress. We could keep all the entries
    // in memory, and maybe we should, but it might get unreasonably big.
    if options.measure_first {
        ui::set_progress_phase("Measure source tree");
        // TODO: Maybe read all entries for the source tree in to memory now, rather than walking it
        // again a second time? But, that'll potentially use memory proportional to tree size, which
        // I'd like to avoid, and also perhaps make it more likely we grumble about files that were
        // deleted or changed while this is running.
        ui::set_bytes_total(source.size()?.file_bytes);
    }

    let target = &options.only_subtree;
    let target_tree: Vec<&str> = target.split('/').collect();

    ui::set_progress_phase("Copying");
    for entry in source.iter_entries()? {
        // Check if this entry is selected for copy
        let subtree: Vec<&str> = entry.apath().split('/').collect();
        // let _: Vec<&str> = entry.is_prefix_of('/');

        let mut to_be_copied: bool = false;

        match target.as_ref() {
            "" => to_be_copied = true,
            _ => {
                // Take the top path from target and match it with entry (accept all subpaths)
                let mut matched: usize = 0;
                if subtree.len() >= target_tree.len() {
                    for (i, _) in target_tree.iter().enumerate() {
                        if target_tree[i].eq(subtree[i]) {
                            matched = matched + 1;
                        }
                    }
                    to_be_copied = matched == target_tree.len();
                }
            }
        }

        if to_be_copied {
            if options.print_filenames {
                crate::ui::println(entry.apath());
            }
            ui::set_progress_file(entry.apath());
            if let Err(e) = match entry.kind() {
                Kind::Dir => {
                    stats.directories += 1;
                    dest.copy_dir(&entry)
                }
                Kind::File => {
                    stats.files += 1;
                    dest.copy_file(&entry, source).map(|s| stats += s)
                }
                Kind::Symlink => {
                    stats.symlinks += 1;
                    dest.copy_symlink(&entry)
                }
                Kind::Unknown => {
                    stats.unknown_kind += 1;
                    // TODO: Perhaps eventually we could backup and restore pipes,
                    // sockets, etc. Or at least count them. For now, silently skip.
                    // https://github.com/sourcefrog/conserve/issues/82
                    continue;
                }
            } {
                ui::show_error(&e);
                stats.errors += 1;
                continue;
            }
            ui::increment_bytes_done(entry.size().unwrap_or(0));
        }
    }
    ui::clear_progress();
    stats += dest.finish()?;
    // TODO: Merge in stats from the tree iter and maybe the source tree?
    Ok(stats)
}
