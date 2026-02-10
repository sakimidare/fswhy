use anyhow::Result;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

mod model;
mod scan;
mod view;

use model::Node;

fn main() -> Result<()> {
    let root_path: PathBuf = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);

    let mut root = Node::scan(root_path)?;

    root.toggle();

    loop {
        let items = view::build_visible_list(&root);

        let max_idx_width = items
            .iter()
            .filter_map(|i| i.index)
            .max()
            .unwrap_or(0)
            .to_string()
            .len();

        println!("\n--- File Tree (Total: {}) ---", items.len());

        for item in &items {
            let prefix = "  ".repeat(item.depth);

            let idx_str = match item.index {
                Some(i) => format!("{:width$}", i, width = max_idx_width),
                None => " ".repeat(max_idx_width),
            };

            let icon = if item.node.is_dir() {
                if item.node.is_expanded() {
                    "[-]"
                } else {
                    "[+]"
                }
            } else {
                "   "
            };

            let size = item.node.size();
            let size_str = if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{:.1} MB", size as f64 / 1024.0 / 1024.0)
            };

            println!(
                "{} {}{} {} ({})",
                idx_str,
                prefix,
                icon,
                item.node
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                size_str
            );
        }

        print!("\n[Index] Toggle Dir | [q] Quit > ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "q" {
            break;
        }

        if let Ok(idx) = input.parse::<usize>() {
            if !view::toggle_by_index(&mut root, idx) {
                println!("Invalid index (Make sure it's a directory index)!");
            }
        }
    }

    Ok(())
}
