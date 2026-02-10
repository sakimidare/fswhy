use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Node {
    path: PathBuf,
    size: u64,
    kind: NodeKind,
}

#[derive(Debug)]
enum NodeKind {
    File,
    Directory(DirProperty),
}

#[derive(Debug)]
pub struct DirProperty {
    children: Vec<Node>,
    expanded: bool,
}

impl Node {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.kind, NodeKind::Directory(_))
    }

    pub fn children(&self) -> Option<&[Node]> {
        match &self.kind {
            NodeKind::Directory(dir) => Some(&dir.children),
            _ => None,
        }
    }

    pub fn scan(path: PathBuf) -> anyhow::Result<Node> {
        let meta = std::fs::metadata(&path)?;

        if meta.is_dir() {
            let mut children = Vec::new();
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                children.push(Node::scan(entry.path())?);
            }

            children.sort_by(|a, b| {
                match (a.is_dir(), b.is_dir()) {
                    (true, false) => std::cmp::Ordering::Less,    // a是目录，b是文件 -> a排前
                    (false, true) => std::cmp::Ordering::Greater, // a是文件，b是目录 -> b排前
                    _ => a.path.cmp(&b.path),                     // 同类按路径名排序
                }
            });

            let total_size: u64 = children.iter().map(|c| c.size).sum();

            Ok(Node {
                path,
                size: total_size,
                kind: NodeKind::Directory(DirProperty {
                    children,
                    expanded: false,
                }),
            })
        } else {
            Ok(Node {
                path,
                size: meta.len(),
                kind: NodeKind::File,
            })
        }
    }

    pub fn is_expanded(&self) -> bool {
        match &self.kind {
            NodeKind::Directory(dir) => dir.expanded,
            _ => false,
        }
    }

    pub fn toggle(&mut self) {
        if let NodeKind::Directory(dir) = &mut self.kind {
            dir.expanded = !dir.expanded;
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut [Node]> {
        match &mut self.kind {
            NodeKind::Directory(dir) => Some(&mut dir.children),
            _ => None,
        }
    }
}
