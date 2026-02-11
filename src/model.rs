//! Data structures representing the file system hierarchy.
//!
//! This module provides the [`Node`] struct, which recursively captures
//! file and directory information, and a [`Node::scan`] method to build
//! the tree from the actual file system.

use crate::model::NodeKind::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// A single entity in the file system tree (either a file or a directory).
///
/// Nodes store essential metadata such as path and size. For directories,
/// the size is the cumulative sum of all descendant nodes.
#[derive(PartialOrd, PartialEq, Debug)]
pub struct Node {
    path: PathBuf,
    size: u64,
    kind: NodeKind,
}

/// Specialized data specific to the type of the [`Node`].
#[derive(PartialOrd, PartialEq, Debug)]
pub enum NodeKind {
    File,
    Directory(DirProperty),
}

#[derive(PartialOrd, PartialEq, Debug)]
pub struct DirProperty {
    children: Vec<Node>,
}

impl DirProperty {
    pub fn children(&self) -> &[Node] {
        &self.children
    }
}

impl Node {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// Recursively scans the filesystem starting from the given path.
    ///
    /// This method builds a tree of [`Node`]s. It calculates the total size
    /// of directories by summing up their children and sorts entries
    /// based on a specific priority:
    /// 1. Directories come before files.
    /// 2. Entries of the same type are sorted alphabetically by path.
    /// Progress is displayed to stderr during scanning:
    /// - Shows progress every 100 items scanned
    /// - Displays detailed statistics for top-level directories
    ///
    /// # Errors
    /// Returns an error if the path does not exist or if permissions are
    /// insufficient to read the directory.
    pub fn scan(path: PathBuf) -> anyhow::Result<Node> {
        // 全局计数器，跨所有层级统计
        static TOTAL_COUNT: AtomicUsize = AtomicUsize::new(0);
        TOTAL_COUNT.store(0, Ordering::Relaxed);

        eprintln!("Scanning {}...", path.display());
        let result = Self::scan_with_progress(path, 0, &TOTAL_COUNT);
        eprintln!();
        result
    }

    /// Internal recursive scan implementation with progress tracking.
    ///
    /// This method is called by [`scan`](Self::scan) and recursively builds
    /// the directory tree while updating a global atomic counter for progress
    /// display.
    ///
    /// # Arguments
    /// * `path` - The filesystem path to scan
    /// * `depth` - Current recursion depth (0 for root)
    /// * `total_count` - Shared atomic counter for tracking total scanned items
    ///
    /// # Progress Display
    /// - Shows incremental progress every 100 items to stderr
    /// - Displays detailed statistics (dir/file count, size, time) for directories
    ///   at depth 0 or 1 to avoid excessive output
    ///
    /// # Error Handling
    /// - Skips inaccessible entries and continues scanning
    /// - Logs errors to stderr only for top-level entries (depth ≤ 1)
    fn scan_with_progress(
        path: PathBuf,
        depth: usize,
        total_count: &AtomicUsize,
    ) -> anyhow::Result<Node> {
        let start = Instant::now();
        let meta = std::fs::metadata(&path)?;

        if meta.is_dir() {
            let mut children = Vec::new();
            let mut file_count = 0;
            let mut dir_count = 0;

            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                let child_path = entry.path();

                match Self::scan_with_progress(child_path, depth + 1, total_count) {
                    Ok(child) => {
                        match child.kind() {
                            File => file_count += 1,
                            Directory(_) => dir_count += 1,
                        }
                        children.push(child);

                        let count = total_count.fetch_add(1, Ordering::Relaxed) + 1;

                        if count % 100 == 0 {
                            eprint!("\rScanned {} items...", count);
                            std::io::Write::flush(&mut std::io::stderr()).ok();
                        }
                    }
                    Err(e) => {
                        if depth <= 1 {
                            eprintln!("\n✗ Skipped: {}", e);
                        }
                    }
                }
            }

            children.sort_by(|a, b| {
                match (&a.kind, &b.kind) {
                    (Directory(_), File) => std::cmp::Ordering::Less,
                    (File, Directory(_)) => std::cmp::Ordering::Greater,
                    _ => a.path.cmp(&b.path),
                }
            });

            let total_size: u64 = children.iter().map(|c| c.size).sum();

            if depth <= 1 {
                eprintln!(
                    "\n✓ {} ({} dirs, {} files, {:.1} MB) in {:.2}s",
                    path.display(),
                    dir_count,
                    file_count,
                    total_size as f64 / 1024.0 / 1024.0,
                    start.elapsed().as_secs_f64(),
                );
            }

            Ok(Node {
                path,
                size: total_size,
                kind: Directory(DirProperty { children }),
            })
        } else {
            Ok(Node {
                path,
                size: meta.len(),
                kind: File,
            })
        }
    }
}